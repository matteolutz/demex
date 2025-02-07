#[derive(Debug)]
pub enum TokenizationError {
    UnknownKeyword(String),
    UnterminatedString,
    UnknownCharacter(char),
}

impl std::fmt::Display for TokenizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenizationError::UnknownKeyword(kw) => write!(f, "Unknown keyword: {}", kw),
            TokenizationError::UnterminatedString => write!(f, "Unterminated string"),
            TokenizationError::UnknownCharacter(c) => write!(f, "Unknown character: {}", c),
        }
    }
}

impl std::error::Error for TokenizationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
