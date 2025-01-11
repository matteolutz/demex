#[derive(Debug)]
pub enum FixtureChannelError {
    FixtureChannelValueWrongVariant(String),
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
        }
    }
}

impl std::error::Error for FixtureChannelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
