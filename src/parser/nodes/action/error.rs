use crate::fixture::handler::error::FixtureHandlerError;

#[derive(Debug)]
pub enum ActionRunError {
    FixtureHandlerError(FixtureHandlerError),
}

impl std::fmt::Display for ActionRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ActionRunError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
