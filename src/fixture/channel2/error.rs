use super::{channel_type::FixtureChannelType, feature::feature_type::FixtureFeatureType};

#[derive(Debug)]
pub enum FixtureChannelError2 {
    Failed,
    ChannelNotFound(FixtureChannelType),
    FeatureNotFound(FixtureFeatureType),
    FeatureConfigNotFound(FixtureFeatureType),
    InvalidFeatureValue(FixtureFeatureType),
    PresetNotFound(u32),

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
            Self::PresetNotFound(preset_id) => {
                write!(f, "Preset with id {} was not found", preset_id)
            }
            Self::NoFeatureFoundFor(feature_group) => {
                write!(f, "No feature found for feature group {}", feature_group)
            }
        }
    }
}
