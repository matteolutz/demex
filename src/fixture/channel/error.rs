use crate::fixture::error::FixtureError;

#[derive(Debug)]
pub enum FixtureChannelError {
    FixtureChannelValueWrongVariant(String),
    FixtureError(FixtureError),
    FixtureChannelValueIsUnset,
}

impl std::fmt::Display for FixtureChannelError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FixtureChannelError::FixtureChannelValueWrongVariant(value) => {
                write!(
                    f,
                    "Fixture channel value has wrong variant, expected {:?}",
                    value
                )
            }
            FixtureChannelError::FixtureError(error) => {
                write!(f, "Fixture error: {}", error)
            }
            Self::FixtureChannelValueIsUnset => {
                write!(f, "Fixture channel value is unset.")
            }
        }
    }
}

impl std::error::Error for FixtureChannelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
