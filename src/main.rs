use anyhow::Error;
use clap::Parser;
use core::fmt;
use file_format::FileFormat;
use fontdue::Font;
use image::ImageError;
use lazy_static::lazy_static;
use log::error;
use resvg;
use std::io;
use std::str::FromStr;
use std::{fs, path::PathBuf};
use thiserror::Error;
use tiny_skia;
use tiny_skia_path;
use usvg;

mod svg;
mod utils;

use svg::code_to_svg;
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

struct FontConfig {
    font: Font,
    font_path: PathBuf,
    font_size: f32,
}

#[derive(Clone, Debug)]
struct Input {
    file_handler: Option<PathBuf>,
    contents: String,
    ext: String,
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

fn parse_to_input(s: &str) -> Result<Input, SiaError> {
    // Convert to path
    let path = PathBuf::from(s);

    // If it is a real path, use that information
    if path.exists() && path.is_file() {
        let ext: String;
        // The extension is the valuable piece of info here. No ext, we need to guess.
        if let Some(extension) = path.extension() {
            ext = extension.to_string_lossy().to_string();
        } else {
            ext = FileFormat::from_file(&path)?.extension().to_string();
        }

        let contents = fs::read_to_string(&path)?;

        Ok(Input {
            file_handler: Some(path),
            ext,
            contents,
        })
    } else {
        // Treat input literally as UTF-8 text
        // Can't help if it's not...
        let bytes = s.as_bytes();
        let ext = FileFormat::from_bytes(bytes).extension().to_string();

        Ok(Input {
            file_handler: None,
            contents: s.into(),
            ext,
        })
    }
}

pub fn parse_rgba8(s: &str) -> Result<rgb::RGBA8, String> {
    // strip leading ‘#’ if present
    let s = s.strip_prefix('#').unwrap_or(s);

    // parse exactly two hex digits into a u8
    fn hex2(pair: &str) -> Result<u8, String> {
        u8::from_str_radix(pair, 16).map_err(|_| format!("`{}` is not valid hex", pair))
    }

    use rgb::RGBA8;
    match s.len() {
        6 => {
            // RRGGBB → (R, G, B, 255)
            let r = hex2(&s[0..2])?;
            let g = hex2(&s[2..4])?;
            let b = hex2(&s[4..6])?;
            Ok(RGBA8::new(r, g, b, 255))
        }
        8 => {
            // RRGGBBAA → (R, G, B, A)
            let r = hex2(&s[0..2])?;
            let g = hex2(&s[2..4])?;
            let b = hex2(&s[4..6])?;
            let a = hex2(&s[6..8])?;
            Ok(RGBA8::new(r, g, b, a))
        }
        _ => Err("hex color must be 6 or 8 digits".into()),
    }
}

#[derive(Parser, Debug)]
#[command(name = "sia", version = "0.2.0", about = "Generate a font preview")]
struct Cli {
    /// Input font file path
    #[arg(short = 'F', long, env = "SIA_FONT")]
    font_path: PathBuf,

    /// Output (image?) file (default: output.png)
    #[arg(short = 'O', long, env = "SIA_OUT_FILE")]
    output: Option<PathBuf>,

    /// Image size WxH
    #[arg(long, env = "SIA_DIMENSIONS")]
    size: Option<Dimensions>,

    /// Font size in px
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
    #[arg(short = 'I', long = "input", value_parser = parse_to_input)]
    input: Input,
}

fn main() {
    env_logger::init();
    if let Err(e) = run() {
        error!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let cli = Cli::parse();

    // Get the font database early to get available fonts
    let mut tree_options = usvg::Options::default();
    tree_options.fontdb_mut().load_system_fonts(); // System fonts should always be loaded? Maybe this is needless
    tree_options.fontdb_mut().load_font_file(&cli.font_path)?;

    // The font name should just be the final component
    let font_data = fs::read(&cli.font_path)?;
    let font = Font::from_bytes(
        font_data,
        fontdue::FontSettings {
            collection_index: 0,
            scale: cli.font_size,
            load_substitutions: true,
        },
    )
    .unwrap();

    // Determine the output file
    let output = cli
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from("output").with_extension("png"));

    // TODO: This only includes three themes, so I'm going to offer an option for users to load their own, just need to see how they're defined.
    let availble_themes = syntect::highlighting::ThemeSet::load_defaults();

    let font_name = font.name().unwrap_or("Times New Roman").to_string();

    let font_name = strip_font_modifier(&font_name);

    // Setup the rendering
    tree_options.dpi = 300.0;
    tree_options.font_family = font_name;
    tree_options.font_size = cli.font_size;

    // Get our svg and final width/height measurements
    let (svg, width, height) = code_to_svg(
        availble_themes.themes.get("base16-ocean.dark").unwrap(),
        &cli.input,
        &FontConfig {
            font,
            font_path: cli.font_path,
            font_size: cli.font_size,
        },
    )?;

    let svg = svg.to_string().replace('\n', "");
    let tree = usvg::Tree::from_str(&svg, &tree_options)?;

    let mut map = tiny_skia::Pixmap::new(width, height).unwrap();

    resvg::render(
        &tree,
        tiny_skia_path::Transform::default(),
        &mut map.as_mut(),
    );

    map.save_png(&output)?;

    Ok(())
}

fn strip_font_modifier(s: &str) -> String {
    // List of modifiers you want to strip if they appear as the last word.
    // You can add or remove entries here as needed.
    let modifiers = [
        "bold",
        "italic",
        "regular",
        "light",
        "medium",
        "semibold",
        "thin",
        "black",
        "book",
        "condensed",
        "extra",
        "ultra",
        "demi",
        "heavy",
        "oblique",
    ];

    // Split on any whitespace, collect words
    let mut parts: Vec<&str> = s.split_whitespace().collect();

    // If there's more than one word, check the last one
    if parts.len() > 1 {
        // Lowercase for case‐insensitive compare
        let last = parts.last().unwrap().to_lowercase();
        if modifiers.contains(&last.as_str()) {
            // Drop the last token and rejoin with single spaces
            parts.pop();
            return parts.join(" ");
        }
    }

    // Otherwise, return original
    s.to_string()
}
