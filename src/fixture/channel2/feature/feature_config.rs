use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{feature_type::FixtureFeatureType, IntoFeatureType};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FixtureFeatureConfig {
    Intensity,
    Zoom,

    ColorRGB,
    ColorMacro { macros: HashMap<u8, [f32; 3]> },
    PositionPanTilt { has_speed: bool },
}

impl IntoFeatureType for FixtureFeatureConfig {
    fn feature_type(&self) -> FixtureFeatureType {
        match self {
            Self::Intensity => FixtureFeatureType::Intensity,
            Self::Zoom => FixtureFeatureType::Zoom,
            Self::ColorRGB => FixtureFeatureType::ColorRGB,
            Self::ColorMacro { .. } => FixtureFeatureType::ColorMacro,
            Self::PositionPanTilt { .. } => FixtureFeatureType::PositionPanTilt,
        }
    }
}
