#[derive(Debug)]
pub enum FixtureHandlerError {
    FixtureNotFound,
    FixtureAlreadyExists,
    FixtureHandlerUpdateError(Box<dyn std::error::Error>),
}

impl std::fmt::Display for FixtureHandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FixtureHandlerError::FixtureNotFound => write!(f, "Fixture not found"),
            FixtureHandlerError::FixtureAlreadyExists => write!(f, "Fixture already exists"),
            FixtureHandlerError::FixtureHandlerUpdateError(e) => {
                write!(f, "Fixture handler update error: {}", e)
            }
        }
    }
}

impl std::error::Error for FixtureHandlerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
