use crate::lexer::token::Token;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(Token, String),
    UnknownAction(String),
    UnexpectedEndOfInput,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken(t, e) => write!(f, "Unexpected token: {:?} ({})", t, e),
            ParseError::UnknownAction(a) => write!(f, "Unknown action: {}", a),
            ParseError::UnexpectedEndOfInput => write!(f, "Unexpected end of input"),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
