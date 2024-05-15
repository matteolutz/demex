use crate::fixture::{error::FixtureError, handler::error::FixtureHandlerError};

#[derive(Debug)]
pub enum ActionRunError {
    FixtureHandlerError(FixtureHandlerError),
    FixtureError(FixtureError),
}

impl std::fmt::Display for ActionRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionRunError::FixtureHandlerError(e) => write!(f, "Fixture handler error: {}", e),
            ActionRunError::FixtureError(e) => write!(f, "Fixture error: {}", e),
        }
    }
}

impl std::error::Error for ActionRunError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
