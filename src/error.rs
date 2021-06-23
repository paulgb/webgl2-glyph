pub enum WebGl2GlyphError {
    WebGlError(String),
    WebGlShaderInfoLog(String),
    WebGlProgramInfoLog(String),
}

impl std::fmt::Display for WebGl2GlyphError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            Self::WebGlError(st) => write!(f, "WebGL Error: {}", &st),
            Self::WebGlProgramInfoLog(st) => write!(f, "WebGL Error linking program: {}", &st),
            Self::WebGlShaderInfoLog(st) => write!(f, "WebGL Error compiling shader: {}", &st),
        }
    }
}

impl std::fmt::Debug for WebGl2GlyphError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}

impl std::error::Error for WebGl2GlyphError {}
