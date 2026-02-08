use serde::{Deserialize, Serialize};

use crate::{
    channel3::feature::feature_group::FixtureChannel3FeatureGroup,
    keyframe_effect::effect_layer::KeyframeEffectLayer,
};

mod pan_tilt_single_origin;
pub use pan_tilt_single_origin::*;

#[derive(Debug, Clone, Serialize, Deserialize, strum::EnumIter)]
pub enum KeyframeEffectPreset {
    /// A Pan-Tilt effect with a single origin point (e.g. Circle, Figure-8, Square)
    PanTiltSingleOrigin(PanTiltSingleOriginEffectPreset),
}

impl KeyframeEffectPreset {
    pub fn feature_group(&self) -> FixtureChannel3FeatureGroup {
        match self {
            Self::PanTiltSingleOrigin(_) => FixtureChannel3FeatureGroup::Position,
        }
    }

    pub fn build_layers(&self) -> Vec<KeyframeEffectLayer> {
        match self {
            Self::PanTiltSingleOrigin(preset) => preset.build_layers(),
        }
    }

    pub fn variant_name(&self) -> &'static str {
        match self {
            Self::PanTiltSingleOrigin(_) => "Pan/Tilt Single Origin",
        }
    }
}
