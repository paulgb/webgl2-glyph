pub enum GlyphAtlasError {
    WebGlError(String),
    WebGlShaderInfoLog(String),
    WebGlProgramInfoLog(String),
    DomError(String),
    InternalError(String),
}

impl std::fmt::Display for GlyphAtlasError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            Self::WebGlError(st) => write!(f, "WebGL Error: {}", &st),
            Self::WebGlProgramInfoLog(st) => write!(f, "WebGL Error linking program: {}", &st),
            Self::WebGlShaderInfoLog(st) => write!(f, "WebGL Error compiling shader: {}", &st),
            Self::DomError(st) => write!(f, "Error interacting with document: {}", &st),
            Self::InternalError(st) => write!(f, "Internal webgl2-glyph-atlas error: {}", &st),
        }
    }
}

impl std::fmt::Debug for GlyphAtlasError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}

pub type Result<T> = std::result::Result<T, GlyphAtlasError>;

impl std::error::Error for GlyphAtlasError {}
