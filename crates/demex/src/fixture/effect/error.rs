use crate::fixture::error::FixtureError;

#[derive(Debug)]
pub enum EffectError {
    EffectNotStarted,
    NoValueForAttribute,
    FixtureError(FixtureError),
}

impl std::error::Error for EffectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for EffectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EffectNotStarted => write!(f, "Effect not started"),
            Self::NoValueForAttribute => write!(f, "No value for attribute"),
            Self::FixtureError(err) => write!(f, "Fixture error: {}", err),
        }
    }
}
