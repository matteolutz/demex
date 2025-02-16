use std::f32;

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::{
    fixture::channel2::feature::{
        feature_type::FixtureFeatureType, feature_value::FixtureFeatureValue,
    },
    utils::math::zero_one_sin,
};

use super::error::EffectError;

pub mod runtime;

#[derive(Debug, Serialize, Deserialize, Clone, EguiProbe)]
pub enum FeatureEffect {
    PositionPanTiltFigureEight {
        placeholder: f32,
    },
    PositionPanTiltCircle {
        pan_size: f32,
        tilt_size: f32,
        pan_offset: f32,
        tilt_offset: f32,
        speed: f32,
    },
}

impl Default for FeatureEffect {
    fn default() -> Self {
        // TODO: change this
        Self::PositionPanTiltFigureEight { placeholder: 0.0 }
    }
}

impl FeatureEffect {
    pub fn feature_type(&self) -> FixtureFeatureType {
        match self {
            FeatureEffect::PositionPanTiltFigureEight { .. }
            | FeatureEffect::PositionPanTiltCircle { .. } => FixtureFeatureType::PositionPanTilt,
        }
    }

    pub fn get_feature_value(&self, t: f64) -> Result<FixtureFeatureValue, EffectError> {
        match self {
            FeatureEffect::PositionPanTiltFigureEight { .. } => {
                let pan = (t as f32 * 2.0 * f32::consts::PI).sin();
                let tilt = (t as f32 * 4.0 * f32::consts::PI).sin();
                Ok(FixtureFeatureValue::PositionPanTilt {
                    pan,
                    tilt,
                    pan_tilt_speed: None,
                })
            }
            FeatureEffect::PositionPanTiltCircle {
                pan_size,
                tilt_size,
                pan_offset,
                tilt_offset,
                speed,
            } => {
                let pan =
                    ((zero_one_sin(t as f32 * speed) * pan_size) + pan_offset).clamp(0.0, 1.0);
                let tilt =
                    ((zero_one_sin(t as f32 * speed) * tilt_size) + tilt_offset).clamp(0.0, 1.0);

                Ok(FixtureFeatureValue::PositionPanTilt {
                    pan,
                    tilt,
                    pan_tilt_speed: None,
                })
            }
        }
    }
}

impl std::fmt::Display for FeatureEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PositionPanTiltCircle { .. } => write!(f, "Circle"),
            Self::PositionPanTiltFigureEight { .. } => write!(f, "Figure 8"),
        }
    }
}
