use crate::lexer::token::Token;

use super::nodes::object::Object;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedVariant(String, Vec<ParseError>),
    UnexpectedToken(Token, String),
    UnknownAction(String),
    NoDefaultActionForObject(Object),
    UnexpectedEndOfInput,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedVariant(e, errors) => {
                write!(f, "{}: {:?}", e, errors)
            }
            ParseError::UnexpectedToken(t, e) => write!(f, "Unexpected token: {:?} ({})", t, e),
            ParseError::UnknownAction(a) => write!(f, "Unknown action: {}", a),
            ParseError::UnexpectedEndOfInput => write!(f, "Unexpected end of input"),
            ParseError::NoDefaultActionForObject(o) => {
                write!(f, "No default action for object: {:?}", o)
            }
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
