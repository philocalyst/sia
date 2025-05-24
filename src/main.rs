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
    line_height: f32,
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

#[derive(Debug, Clone, Copy)]
struct Alpha(f32);

impl Alpha {
    fn to_u8(self) -> u8 {
        (self.0.clamp(0.0, 1.0) * 255.0).round() as u8
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

fn parse_input(s: &str) -> Result<Input, SiaError> {
    // Convert to path
    let path = PathBuf::from(s);

    // If it is a real path, use that information
    if path.exists() && path.is_file() {
        let ext: String;
        // The extension is the valuable piece of info here. No ext, we need to guess.
        if let Some(ext) = path.extension() {
            ext = ext.to_string_lossy().into();
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

fn run() -> Result<(), SiaError> {}
