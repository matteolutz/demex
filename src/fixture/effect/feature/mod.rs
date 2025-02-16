use std::f32;

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::channel2::feature::{
    feature_type::FixtureFeatureType, feature_value::FixtureFeatureValue,
};

use super::error::EffectError;

pub mod runtime;

#[derive(Debug, Serialize, Deserialize, Clone, EguiProbe)]
pub enum FeatureEffect {
    PositionPanTiltFigureEight {
        pan_size: f32,
        tilt_size: f32,
        pan_center: f32,
        tilt_center: f32,
        speed: f32,
    },
    PositionPanTiltEllipse {
        pan_size: f32,
        tilt_size: f32,
        pan_center: f32,
        tilt_center: f32,
        speed: f32,
    },
    PositionPanTiltRect {
        pan_size: f32,
        tilt_size: f32,
        pan_center: f32,
        tilt_center: f32,
        speed: f32,
    },
}

impl Default for FeatureEffect {
    fn default() -> Self {
        // TODO: change this
        Self::PositionPanTiltFigureEight {
            pan_size: 0.0,
            tilt_size: 0.0,
            pan_center: 0.5,
            tilt_center: 0.5,
            speed: 0.0,
        }
    }
}

impl FeatureEffect {
    pub fn feature_type(&self) -> FixtureFeatureType {
        match self {
            Self::PositionPanTiltFigureEight { .. }
            | Self::PositionPanTiltEllipse { .. }
            | Self::PositionPanTiltRect { .. } => FixtureFeatureType::PositionPanTilt,
        }
    }

    pub fn get_feature_value(&self, t: f64) -> Result<FixtureFeatureValue, EffectError> {
        match self {
            Self::PositionPanTiltRect { .. } => Err(EffectError::EffectNotStarted),
            Self::PositionPanTiltFigureEight {
                pan_size,
                tilt_size,
                pan_center,
                tilt_center,
                speed,
            } => {
                // TODO: should we multiply pan or tilt?
                let pan = (f32::sin(t as f32 * speed * 2.0)) * (pan_size / 2.0) + pan_center;
                let tilt = (f32::sin(t as f32 * speed)) * (tilt_size / 2.0) + tilt_center;

                Ok(FixtureFeatureValue::PositionPanTilt {
                    pan,
                    tilt,
                    pan_tilt_speed: None,
                })
            }
            Self::PositionPanTiltEllipse {
                pan_size,
                tilt_size,
                pan_center,
                tilt_center,
                speed,
            } => {
                let pan = (f32::sin(t as f32 * speed)) * (pan_size / 2.0) + pan_center;
                let tilt = (f32::sin(t as f32 * speed)) * (tilt_size / 2.0) + tilt_center;

                Ok(FixtureFeatureValue::PositionPanTilt {
                    pan: pan.clamp(0.0, 1.0),
                    tilt: tilt.clamp(0.0, 1.0),
                    pan_tilt_speed: None,
                })
            }
        }
    }
}

impl std::fmt::Display for FeatureEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PositionPanTiltEllipse { .. } => write!(f, "Circle"),
            Self::PositionPanTiltFigureEight { .. } => write!(f, "Figure 8"),
            Self::PositionPanTiltRect { .. } => write!(f, "Rect"),
        }
    }
}
