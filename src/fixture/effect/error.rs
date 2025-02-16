use crate::fixture::channel2::error::FixtureChannelError2;

#[derive(Debug)]
pub enum EffectError {
    EffectNotStarted,
    FixtureChannelError(FixtureChannelError2),
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
            Self::FixtureChannelError(err) => write!(f, "Fixture channel error: {}", err),
        }
    }
}
