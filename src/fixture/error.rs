use super::channel2::{
    channel_type::FixtureChannelType, error::FixtureChannelError2,
    feature::feature_type::FixtureFeatureType,
};

#[derive(Debug)]
pub enum FixtureError {
    ChannelNotFound(FixtureChannelType),
    FeatureNotFound(FixtureFeatureType),
    ChannelValueNotFound(FixtureChannelType),
    NoChannelValueSourceFound,
    EmptyPatch,
    DuplicateChannelType,
    InvalidDataLength,
    NoFunctionAccess,
    FixtureChannelError2(FixtureChannelError2),
    FixtureTypeNotFound(String),
    FixtureTypeModeNotFound(String, u32),
}

impl std::fmt::Display for FixtureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChannelNotFound(s) => write!(f, "Channel ({:?}) not found", s),
            Self::FeatureNotFound(feature) => write!(f, "Feature ({:?}) not found", feature),
            Self::ChannelValueNotFound(channel_type) => {
                write!(f, "Channel value for type {:?} not found", channel_type)
            }
            Self::NoChannelValueSourceFound => write!(f, "No channel value source found"),
            Self::EmptyPatch => write!(f, "Patch is empty"),
            Self::DuplicateChannelType => write!(f, "Duplicate channel type"),
            Self::InvalidDataLength => write!(f, "Invalid data length"),
            Self::FixtureChannelError2(e) => write!(f, "Fixture channel error: {}", e),
            Self::NoFunctionAccess => write!(f, "Tried to access values for a NoFunction channel"),
            Self::FixtureTypeNotFound(s) => write!(f, "Fixture type ({}) not found", s),
            Self::FixtureTypeModeNotFound(s, id) => {
                write!(f, "Fixture type mode ({}) with id ({}) not found", s, id)
            }
        }
    }
}

impl std::error::Error for FixtureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
