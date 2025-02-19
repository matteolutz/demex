use std::f32;

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::{
    fixture::channel2::feature::{
        feature_type::FixtureFeatureType, feature_value::FixtureFeatureValue,
    },
    utils::color::hsl_to_rgb,
};

use super::error::EffectError;

pub mod runtime;

#[derive(Debug, Serialize, Deserialize, Clone, EguiProbe)]
pub enum FeatureEffect {
    IntensitySine {
        speed: f32,
    },

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

    ColorRGBHueRotate {
        hue_size: f32,
        hue_center: f32,
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
            Self::ColorRGBHueRotate { .. } => FixtureFeatureType::ColorRGB,
            Self::IntensitySine { .. } => FixtureFeatureType::Intensity,
        }
    }

    pub fn get_feature_value(
        &self,
        t: f64,
        phase_offset: f32,
    ) -> Result<FixtureFeatureValue, EffectError> {
        match self {
            Self::IntensitySine { speed } => {
                let intensity = f32::sin(
                    t as f32 * speed + (3.0 * f32::consts::FRAC_PI_2) - phase_offset.to_radians(),
                ) * 0.5
                    + 0.5;

                Ok(FixtureFeatureValue::Intensity { intensity })
            }
            Self::PositionPanTiltRect { .. } => Err(EffectError::EffectNotStarted),
            Self::PositionPanTiltFigureEight {
                pan_size,
                tilt_size,
                pan_center,
                tilt_center,
                speed,
            } => {
                // TODO: should we multiply pan or tilt?
                let pan = (f32::sin(t as f32 * speed * 2.0 - phase_offset.to_radians()))
                    * (pan_size / 2.0)
                    + pan_center;
                let tilt = (f32::sin(t as f32 * speed - phase_offset.to_radians()))
                    * (tilt_size / 2.0)
                    + tilt_center;

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
                let pan = (f32::sin(t as f32 * speed - phase_offset.to_radians()))
                    * (pan_size / 2.0)
                    + pan_center;
                let tilt = (f32::sin(t as f32 * speed - phase_offset.to_radians()))
                    * (tilt_size / 2.0)
                    + tilt_center;

                Ok(FixtureFeatureValue::PositionPanTilt {
                    pan: pan.clamp(0.0, 1.0),
                    tilt: tilt.clamp(0.0, 1.0),
                    pan_tilt_speed: None,
                })
            }

            Self::ColorRGBHueRotate {
                hue_size,
                hue_center,
                speed,
            } => {
                let hue = (f32::sin(t as f32 * speed - phase_offset.to_radians()))
                    * (hue_size / 2.0)
                    + hue_center;
                let [r, g, b] = hsl_to_rgb([hue, 1.0, 0.5]);

                Ok(FixtureFeatureValue::ColorRGB { r, g, b })
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

            Self::ColorRGBHueRotate { .. } => write!(f, "HueRotate"),
            Self::IntensitySine { .. } => write!(f, "IntensitySine"),
        }
    }
}
