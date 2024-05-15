#[derive(Debug)]
pub enum FixtureError {
    ChannelNotFound(Option<String>),
    EmptyPatch,
    DuplicateChannelType,
}

impl std::fmt::Display for FixtureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChannelNotFound(s) => write!(f, "Channel ({:?}) not found", s),
            Self::EmptyPatch => write!(f, "Patch is empty"),
            Self::DuplicateChannelType => write!(f, "Duplicate channel type"),
        }
    }
}

impl std::error::Error for FixtureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
