#[derive(Debug)]
pub enum TokenizationError {
    UnknownKeyword(String),
}

impl std::fmt::Display for TokenizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenizationError::UnknownKeyword(kw) => write!(f, "Unknown keyword: {}", kw),
        }
    }
}

impl std::error::Error for TokenizationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
