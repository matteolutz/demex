use crate::fixture::presets::error::PresetHandlerError;

use super::{channel_type::FixtureChannelType, feature::feature_type::FixtureFeatureType};

#[derive(Debug)]
pub enum FixtureChannelError2 {
    Failed,
    ChannelNotFound(FixtureChannelType),
    FineChannelNotFound(FixtureChannelType),
    FeatureNotFound(FixtureFeatureType),
    FeatureConfigNotFound(FixtureFeatureType),
    InvalidFeatureValue(FixtureFeatureType),
    PresetHandlerError(Box<PresetHandlerError>),

    NoFeatureFoundFor(u32),
}

impl std::error::Error for FixtureChannelError2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for FixtureChannelError2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Failed => write!(f, "Failed"),
            Self::ChannelNotFound(channel_type) => {
                write!(f, "Channel with type {:?} not found", channel_type)
            }
            Self::FineChannelNotFound(channel_type) => {
                write!(f, "Fine channel with type {:?} not found", channel_type)
            }
            Self::FeatureNotFound(feature_type) => {
                write!(f, "Feature type {:?} not found", feature_type)
            }
            Self::FeatureConfigNotFound(feature_type) => {
                write!(f, "Feature config for type {:?} not found", feature_type)
            }
            Self::InvalidFeatureValue(feature_type) => {
                write!(
                    f,
                    "Invalid feature value for feature of type {:?}",
                    feature_type
                )
            }
            Self::PresetHandlerError(err) => write!(f, "Preset handler error: {}", err),
            Self::NoFeatureFoundFor(feature_group) => {
                write!(f, "No feature found for feature group {}", feature_group)
            }
        }
    }
}
