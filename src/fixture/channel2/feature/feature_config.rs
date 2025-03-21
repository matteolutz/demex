use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::fixture::channel2::{
    channel_type::FixtureChannelType, color::color_gel::ColorGel, gobo::GoboMacro,
};

use super::{feature_type::FixtureFeatureType, wheel::WheelFeatureConfig, IntoFeatureType};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FixtureFeatureConfig {
    SingleValue {
        channel_type: FixtureChannelType,
        is_fine: bool,
    },

    ColorRGB {
        is_fine: bool,
    },
    ColorWheel {
        wheel_config: WheelFeatureConfig<ColorGel>,
    },

    GoboWheel {
        wheel_config: WheelFeatureConfig<GoboMacro>,
    },

    PositionPanTilt {
        is_fine: bool,
        has_speed: bool,
    },

    ToggleFlags {
        toggle_flags: Vec<HashMap<String, u8>>,
    },
}

impl IntoFeatureType for FixtureFeatureConfig {
    fn feature_type(&self) -> FixtureFeatureType {
        match self {
            &Self::SingleValue { channel_type, .. } => {
                FixtureFeatureType::SingleValue { channel_type }
            }

            Self::ColorRGB { .. } => FixtureFeatureType::ColorRGB,
            Self::ColorWheel { .. } => FixtureFeatureType::ColorWheel,

            Self::GoboWheel { .. } => FixtureFeatureType::GoboWheel,

            Self::PositionPanTilt { .. } => FixtureFeatureType::PositionPanTilt,

            Self::ToggleFlags { .. } => FixtureFeatureType::ToggleFlags,
        }
    }
}
