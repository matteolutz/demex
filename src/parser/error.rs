use itertools::Itertools;

use crate::lexer::token::Token;

use super::nodes::object::{Object, ObjectError};

#[derive(Debug)]
pub enum ParseError {
    UnexpectedVariant(Vec<(String, ParseError)>),
    UnexpectedToken(Token, String),
    UnexpectedTokenAlternatives(Token, Vec<&'static str>),
    UnknownAction(String),
    NoDefaultActionForObject(Object),
    ObjectError(ObjectError),
    UnexpectedEndOfInput,

    UnexpectedArgs(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedVariant(variants) => {
                write!(
                    f,
                    "Expected {}:\n\t{}",
                    variants
                        .iter()
                        .enumerate()
                        .map(|(idx, (v, _))| format!("{} ({})", v, idx + 1))
                        .join(" or "),
                    variants
                        .iter()
                        .enumerate()
                        .map(|(idx, (_, e))| format!("{}: {}", idx + 1, e))
                        .join("\n\t")
                )
            }
            ParseError::UnexpectedToken(t, e) => write!(f, "Unexpected token: {:?} ({})", t, e),
            ParseError::UnexpectedTokenAlternatives(t, e) => {
                write!(
                    f,
                    "Unexpected token: {:?} (expected {})",
                    t,
                    e.iter().join(" or ")
                )
            }
            ParseError::UnknownAction(a) => write!(f, "Unknown action: {}", a),
            ParseError::UnexpectedEndOfInput => write!(f, "Unexpected end of input"),
            ParseError::NoDefaultActionForObject(o) => {
                write!(f, "No default action for object: {:?}", o)
            }
            ParseError::ObjectError(e) => write!(f, "Object error: {}", e),

            ParseError::UnexpectedArgs(e) => write!(f, "Unexpected args: {}", e),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
