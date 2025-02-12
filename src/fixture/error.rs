use super::channel::{error::FixtureChannelError, FixtureChannel};

#[derive(Debug)]
pub enum FixtureError {
    ChannelNotFound(Option<String>),
    ChannelValueNotFound(u16),
    NoChannelValueSourceFound,
    EmptyPatch,
    DuplicateChannelType,
    InvalidDataLength,
    NoFunctionAccess,
    FixtureChannelError(Box<FixtureChannelError>),
}

impl std::fmt::Display for FixtureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChannelNotFound(s) => write!(f, "Channel ({:?}) not found", s),
            Self::ChannelValueNotFound(id) => write!(
                f,
                "Channel value ({}) not found",
                FixtureChannel::name_by_id(*id)
            ),
            Self::NoChannelValueSourceFound => write!(f, "No channel value source found"),
            Self::EmptyPatch => write!(f, "Patch is empty"),
            Self::DuplicateChannelType => write!(f, "Duplicate channel type"),
            Self::InvalidDataLength => write!(f, "Invalid data length"),
            Self::FixtureChannelError(e) => write!(f, "Fixture channel error: {}", e),
            Self::NoFunctionAccess => write!(f, "Tried to access values for a NoFunction channel"),
        }
    }
}

impl std::error::Error for FixtureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
