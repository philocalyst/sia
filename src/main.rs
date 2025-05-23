// External crates
use clap::Parser;
use image::{ImageError, Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use rgb::RGBA8;
use rusttype::{Font, Point, Scale};
use thiserror::Error;

// Standard library imports
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::{fmt, fs, io};

mod svg;
mod utils;

use utils::{get_canvas_size, get_text_info};

// The latin codes I know about. Compiled very ad-hoc, so if there are any missing please let me know. I would value some good advice here
lazy_static! {
    static ref LATIN_CODES: Vec<&'static str> = vec![
        "aa", "af", "ay", "bi", "br", "bs", "ca", "ch", "co", "cs", "cy", "da", "de", "en", "eo",
        "es", "et", "eu", "fi", "fj", "fo", "fr", "fur", "fy", "gd", "gl", "gv", "ho", "hr", "hu",
        "ia", "id", "ie", "io", "is", "it", "ki", "kl", "la", "lb", "lt", "lv", "mg", "mh", "mt",
        "nb", "nds", "nl", "nn", "no", "nr", "nso", "ny", "oc", "om", "pl", "pt", "rm", "ro", "se",
        "sk", "sl", "sma", "smj", "smn", "so", "sq", "ss", "st", "sv", "sw", "tk", "tl", "tn",
        "tr", "ts", "uz", "vo", "vot", "wa", "wen", "wo", "xh", "yap", "zu", "an", "crh", "csb",
        "fil", "hsb", "ht", "jv", "kj", "ku-tr", "kwm", "lg", "li", "ms", "na", "ng", "pap-an",
        "pap-aw", "rn", "rw", "sc", "sg", "sn", "su", "ty", "za", "agr", "ayc", "bem", "dsb",
        "lij", "mfe", "mjw", "nhn", "niu", "sgs", "szl", "tpi", "unm", "wae", "yuw",
    ];
}

struct FontConfig<'a> {
    font: Font<'a>,
    scale: Scale,
    line_height: f32,
    font_name: String,
    font_size: f32,
}

use file_format::FileFormat;
#[derive(Clone, Debug)]
struct Input {
    file_handler: Option<PathBuf>,
    contents: Content,
    kind: FileFormat,
}

#[derive(Clone, Debug)]
struct Content {
    source: String,
    line_count: u32,
    largest_line_length: u32,
}

#[derive(Debug, Clone, Copy)]
struct Dimensions {
    width: u32,
    height: u32,
}

impl FromStr for Dimensions {
    type Err = SiaError;
    fn from_str(s: &str) -> Result<Self, SiaError> {
        let mut parts = s.split('x');
        let w = parts
            .next()
            .and_then(|p| p.parse().ok())
            .ok_or_else(|| SiaError::InvalidConfig("size".into()))?;
        let h = parts
            .next()
            .and_then(|p| p.parse().ok())
            .ok_or_else(|| SiaError::InvalidConfig("size".into()))?;
        Ok(Dimensions {
            width: w,
            height: h,
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct Alpha(f32);

impl Alpha {
    fn to_u8(self) -> u8 {
        (self.0.clamp(0.0, 1.0) * 255.0).round() as u8
    }
}

impl FromStr for Alpha {
    type Err = SiaError;
    fn from_str(s: &str) -> Result<Self, SiaError> {
        let v: f32 = s
            .parse()
            .map_err(|_| SiaError::InvalidConfig("alpha".into()))?;
        Ok(Alpha(v.clamp(0.0, 1.0)))
    }
}

impl fmt::Display for Alpha {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Error)]
enum SiaError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("XML Parse error: {0}")]
    XmlParseError(String),

    #[error("Invalid SVG: {0}")]
    InvalidSvg(String),

    #[error("Image error: {0}")]
    Image(#[from] ImageError),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Error While Parsing: {0}")]
    Parse(String),

    #[error("Font load failure: {0}")]
    FontLoad(String),

    #[error("Latin‐script detection error: {0}")]
    LatinDetect(String),

    #[error("Font‐name detection error: {0}")]
    FontNameDetect(String),

    #[error("Sia Error: {0}")]
    Message(String),
}

impl From<String> for SiaError {
    fn from(s: String) -> Self {
        SiaError::Message(s)
    }
}

fn parse_input(s: &str) -> Result<Input, SiaError> {
    let path = PathBuf::from(s);
    if path.exists() && path.is_file() {
        // Guess the file format
        let kind = FileFormat::from_file(&path)?;
        // Convert bytes → String, replacing invalid UTF-8
        let contents = fs::read_to_string(&path)?;

        let (max_chars, line_count) = get_text_info(&contents);

        Ok(Input {
            file_handler: Some(path),
            kind,
            contents: Content {
                source: contents,
                line_count,
                largest_line_length: max_chars,
            },
        })
    } else {
        // Treat input literally as UTF-8 text
        let bytes = s.as_bytes();
        let kind = FileFormat::from_bytes(bytes);
        let (max_chars, line_count) = get_text_info(&s);

        Ok(Input {
            file_handler: None,
            contents: Content {
                source: s.into(),
                line_count,
                largest_line_length: max_chars,
            },
            kind,
        })
    }
}

fn parse_rgba8(hex_code: &str) -> Result<RGBA8, SiaError> {
    // strip leading ‘#’ if any
    let hex = hex_code.trim().strip_prefix('#').unwrap_or(hex_code);

    match hex.len() {
        6 => {
            // RRGGBB => alpha = 0xFF
            let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
            let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
            let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;
            Ok(RGBA8::new(r, g, b, u8::MAX))
        }
        8 => {
            // RRGGBBAA
            let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
            let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
            let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;
            let a = u8::from_str_radix(&hex[6..8], 16).map_err(|e| e.to_string())?;
            Ok(RGBA8::new(r, g, b, a))
        }
        _ => Err(SiaError::Parse(format!(
            "invalid color `{}`, expected `#RRGGBB` or `#RRGGBBAA`",
            hex_code
        ))),
    }
}

#[derive(Parser, Debug)]
#[command(name = "sia", version = "0.2.0", about = "Generate a font preview")]
struct Cli {
    /// Input font file path
    #[arg(short = 'F', long, env = "SIA_FONT")]
    font_path: PathBuf,

    /// Output image file (default: <font>.png)
    #[arg(short = 'O', long, env = "SIA_OUT_FILE")]
    output: Option<PathBuf>,

    /// Image size WxH
    #[arg(long, env = "SIA_DIMENSIONS")]
    size: Option<Dimensions>,

    /// Font size in px, or relative units (%)
    #[arg(long, env = "SIA_FONT_SIZE")]
    font_size: f32,

    /// Background color
    #[arg(long, default_value = "#FFFFFF", env = "SIA_BG_COLOR", value_parser = parse_rgba8)]
    bg_color: rgb::RGBA8,

    /// Text color
    #[arg(long, default_value = "#000000", env = "SIA_FG_COLOR", value_parser = parse_rgba8)]
    fg_color: rgb::RGBA8,

    /// Background alpha
    #[arg(long, default_value_t = Alpha(1.0), env = "SIA_BG_ALPHA")]
    bg_alpha: Alpha,

    /// Text alpha
    #[arg(long, default_value_t = Alpha(1.0), env = "SIA_FG_ALPHA")]
    fg_alpha: Alpha,

    /// Text or file to render (\\n separated).
    #[arg(short = 'I', long = "input", value_parser = parse_input)]
    input: Input,
}

fn main() {
    env_logger::init();
    if let Err(e) = run() {
        error!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), SiaError> {
    let cli = Cli::parse();
    let font_path = cli.font_path;

    // Detect font family name
    let family = get_font_name(&font_path)?;
    info!("Detected font family: {}", family);

    // Derive the true font_size
    let font_size: f32 = cli.font_size;

    // Read font bytes
    let font_data = fs::read(&font_path)?;
    let scale = Scale::uniform(font_size);

    let font =
        Font::try_from_vec(font_data).ok_or_else(|| SiaError::FontLoad("invalid font".into()))?;

    let v_metrics = font.v_metrics(scale);
    let line_height = (v_metrics.ascent - v_metrics.descent + v_metrics.line_gap).ceil();

    let full_font = FontConfig {
        font,
        scale,
        line_height,
        font_name: family,
        font_size,
    };

    // Determine the output file
    let output = cli
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from("output").with_extension("png"));

    let available_themes = syntect::highlighting::ThemeSet::load_defaults();
    let result = code_to_svg(
        available_themes.themes.get("base16-ocean.dark").unwrap(),
        &cli.input,
        &full_font,
    );

    use resvg;
    use usvg;

    let tree =
        usvg::Tree::from_str(&result.unwrap().to_string(), &usvg::Options::default()).unwrap();

    resvg::render(&tree);
    // Build the background canvas
    let (size, advance_width) = get_canvas_size(
        None,
        cli.input.contents.largest_line_length,
        cli.input.contents.line_count,
        &full_font,
    );

    // Script‐support check
    match detect_latin_support(&font_path) {
        Ok(false) => warn!("Font has not declared Latin‐script support."),
        Err(e) => warn!("Could not detect script support: {}", e),
        _ => {}
    }

    info!("Saving to {:?}", output);

    info!("Done.");
    Ok(())
}

#[cfg(unix)]
fn get_font_name(path: &Path) -> Result<String, SiaError> {
    let p = path
        .to_str()
        .ok_or_else(|| SiaError::FontNameDetect("invalid path".into()))?;
    let out = Command::new("fc-scan")
        .args(["--format", "%{family}", p])
        .output()
        .map_err(|e| SiaError::FontNameDetect(e.to_string()))?;
    if !out.status.success() {
        return Err(SiaError::FontNameDetect(format!(
            "fc-scan failed: {:?}",
            out.status
        )));
    }
    let fam = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if fam.is_empty() {
        return Ok("NA".into());
    }
    // pick first segment before ',' or space
    let short = if fam.contains(',') {
        fam.split(',').next().unwrap().trim()
    } else if fam.contains(' ') {
        fam.split_ascii_whitespace().next().unwrap()
    } else {
        // first proper noun: uppercase + following lowercase
        let chars = fam.char_indices();
        let mut result = None;
        for (i, c) in chars {
            if c.is_uppercase() {
                let mut end = i + c.len_utf8();
                for (j, d) in fam[end..].char_indices() {
                    if d.is_uppercase() {
                        break;
                    }
                    end = i + j + d.len_utf8();
                }
                result = Some(&fam[i..end]);
                break;
            }
        }
        result.unwrap_or(&fam)
    };
    Ok(short.to_string())
}

#[cfg(windows)]
fn get_font_name(path: &Path) -> Result<String, SiaError> {
    let p = path
        .to_str()
        .ok_or_else(|| SiaError::FontNameDetect("invalid path".into()))?;
    let cmd = format!(
        "[System.Drawing.FontFamily]::Families \
         | Where-Object {{ $_.GetName(0) -eq (\"{}\") }} \
         | Select-Object -ExpandProperty Name",
        p
    );
    let out = Command::new("powershell")
        .args(&["-Command", &cmd])
        .output()
        .map_err(|e| SiaError::FontNameDetect(e.to_string()))?;
    if !out.status.success() {
        return Err(SiaError::FontNameDetect(format!(
            "powershell failed: {:?}",
            out.status
        )));
    }
    let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok(if name.is_empty() { "NA".into() } else { name })
}

#[cfg(not(any(unix, windows)))]
fn get_font_name(_: &Path) -> Result<String, SiaError> {
    Ok("NA".into()) // I don't know what to do here, looking for advice. Would this be BSD-based systems?
}

#[cfg(unix)]
fn detect_latin_support(path: &Path) -> Result<bool, SiaError> {
    let out = Command::new("fc-scan")
        .args([
            "--format",
            "%{lang}",
            path.to_str()
                .ok_or_else(|| SiaError::LatinDetect("invalid font path".into()))?,
        ])
        .output()
        .map_err(|e| SiaError::LatinDetect(e.to_string()))?;
    let langs = String::from_utf8_lossy(&out.stdout);
    for code in langs
        .split(|c: char| c == ',' || c == '|' || c.is_whitespace())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if LATIN_CODES.contains(&code) {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(windows)]
fn detect_latin_support(_path: &Path) -> Result<bool, SiaError> {
    // skipping on Windows
    Ok(true)
}
