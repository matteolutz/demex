use crate::fixture::{channel2::error::FixtureChannelError2, error::FixtureError};

#[derive(Debug)]
pub enum FixtureHandlerError {
    FixtureNotFound(u32),
    FixtureAlreadyExists,
    FixtureHandlerUpdateError(Box<dyn std::error::Error>),
    FixtureError(FixtureError),
    FixtureAddressOverlap(u16, u16, u16),
    FixtureChannelError(FixtureChannelError2),
}

impl std::fmt::Display for FixtureHandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FixtureHandlerError::FixtureNotFound(fixture) => {
                write!(f, "Fixture {} not found", fixture)
            }
            FixtureHandlerError::FixtureAlreadyExists => write!(f, "Fixture already exists"),
            FixtureHandlerError::FixtureHandlerUpdateError(e) => {
                write!(f, "Fixture handler update error: {}", e)
            }
            FixtureHandlerError::FixtureError(e) => write!(f, "Fixture error: {}", e),
            FixtureHandlerError::FixtureAddressOverlap(universe, start, end) => {
                write!(
                    f,
                    "Fixture address overlap (U{}): {} - {}",
                    universe, start, end
                )
            }
            FixtureHandlerError::FixtureChannelError(e) => {
                write!(f, "Fixture channel error: {}", e)
            }
        }
    }
}

impl std::error::Error for FixtureHandlerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
