use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::fixture::channel2::{channel_type::FixtureChannelType, color::color_gel::ColorGel};

use super::{feature_type::FixtureFeatureType, IntoFeatureType};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FixtureFeatureConfig {
    Intensity {
        is_fine: bool,
    },

    SingleValue {
        channel_type: FixtureChannelType,
        is_fine: bool,
    },

    Zoom {
        is_fine: bool,
    },
    Focus {
        is_fine: bool,
    },

    Shutter,

    ColorRGB {
        is_fine: bool,
    },
    ColorMacro {
        macros: Vec<(u8, ColorGel)>,
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
            Self::Intensity { .. } => FixtureFeatureType::Intensity,
            &Self::SingleValue { channel_type, .. } => {
                FixtureFeatureType::SingleValue { channel_type }
            }
            Self::Zoom { .. } => FixtureFeatureType::Zoom,
            Self::Focus { .. } => FixtureFeatureType::Focus,
            Self::Shutter => FixtureFeatureType::Shutter,
            Self::ColorRGB { .. } => FixtureFeatureType::ColorRGB,
            Self::ColorMacro { .. } => FixtureFeatureType::ColorMacro,
            Self::PositionPanTilt { .. } => FixtureFeatureType::PositionPanTilt,
            Self::ToggleFlags { .. } => FixtureFeatureType::ToggleFlags,
        }
    }
}
