use std::f32;

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};
use sine::SineVariant;

pub mod runtime;
pub mod sine;

#[derive(Debug, Serialize, Deserialize, Clone, EguiProbe)]
pub enum FeatureEffect {
    IntensitySine {
        sine_variant: SineVariant,
    },

    PositionPanTiltFigureEight {
        pan_size: f32,
        tilt_size: f32,
        pan_center: f32,
        tilt_center: f32,
    },

    PositionPanTiltEllipse {
        pan_size: f32,
        tilt_size: f32,
        pan_center: f32,
        tilt_center: f32,
        sine_variant: SineVariant,
    },

    PositionPanTiltRect {
        pan_size: f32,
        tilt_size: f32,
        pan_center: f32,
        tilt_center: f32,
    },

    ColorRGBHueRotate {
        hue_size: f32,
        hue_center: f32,
    },
}

impl Default for FeatureEffect {
    fn default() -> Self {
        Self::IntensitySine {
            sine_variant: SineVariant::default(),
        }
    }
}

impl FeatureEffect {
    /*
    pub fn default_for(feature_group: DefaultFeatureGroup) -> Option<Self> {
        match feature_group {
            DefaultFeatureGroup::Intensity => Some(Self::IntensitySine {
                sine_variant: SineVariant::default(),
            }),
            DefaultFeatureGroup::Position => Some(Self::PositionPanTiltEllipse {
                pan_size: 1.0,
                tilt_size: 1.0,
                pan_center: 0.5,
                tilt_center: 0.5,
                sine_variant: SineVariant::default(),
            }),
            DefaultFeatureGroup::Color => Some(Self::ColorRGBHueRotate {
                hue_size: 1.0,
                hue_center: 0.5,
            }),
            _ => None,
        }
    }

    pub fn feature_type(&self) -> FixtureFeatureType {
        match self {
            Self::PositionPanTiltFigureEight { .. }
            | Self::PositionPanTiltEllipse { .. }
            | Self::PositionPanTiltRect { .. } => FixtureFeatureType::PositionPanTilt,
            Self::ColorRGBHueRotate { .. } => FixtureFeatureType::ColorRGB,
            Self::IntensitySine { .. } => FixtureFeatureType::SingleValue {
                channel_type: FixtureChannelType::Intensity,
            },
        }
    }

    pub fn get_feature_value(
        &self,
        t: f64,
        phase_offset_deg: f32,
        speed: f32,
    ) -> Result<FixtureFeatureValue, EffectError> {
        match self {
            Self::IntensitySine { sine_variant } => {
                let intensity =
                    sine_variant.apply(t as f32 * speed - phase_offset_deg.to_radians());

                Ok(FixtureFeatureValue::SingleValue {
                    channel_type: FixtureChannelType::Intensity,
                    value: intensity,
                })
            }
            Self::PositionPanTiltRect { .. } => Err(EffectError::EffectNotStarted),
            Self::PositionPanTiltFigureEight {
                pan_size,
                tilt_size,
                pan_center,
                tilt_center,
            } => {
                // TODO: should we multiply pan or tilt?
                let pan = (f32::sin(t as f32 * speed * 2.0 - phase_offset_deg.to_radians()))
                    * (pan_size / 2.0)
                    + pan_center;
                let tilt = (f32::sin(t as f32 * speed - phase_offset_deg.to_radians()))
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
                sine_variant,
            } => {
                let pan = sine_variant.apply(t as f32 * speed - phase_offset_deg.to_radians())
                    * (pan_size / 2.0)
                    + pan_center;

                let tilt = sine_variant.apply(t as f32 * speed - phase_offset_deg.to_radians())
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
            } => {
                let hue = (f32::sin(t as f32 * speed - phase_offset_deg.to_radians()))
                    * (hue_size / 2.0)
                    + hue_center;
                let [r, g, b] = hsl_to_rgb([hue, 1.0, 0.5]);

                Ok(FixtureFeatureValue::ColorRGB { r, g, b })
            }
        }
    }
    */

    pub fn get_attributes(&self) -> Vec<&str> {
        match self {
            Self::IntensitySine { .. } => vec!["Dimmer"],
            Self::PositionPanTiltFigureEight { .. } => vec!["Pan", "Tilt"],
            _ => todo!(),
        }
    }

    pub fn get_attribute_value(
        &self,
        attribute_name: &str,
        t: f64,
        phase_offset_deg: f32,
        speed: f32,
    ) -> Option<f32> {
        match self {
            Self::IntensitySine { sine_variant } => {
                if attribute_name != "Dimmer" {
                    None
                } else {
                    Some(sine_variant.apply(t as f32 * speed - phase_offset_deg.to_radians()))
                }
            }
            Self::PositionPanTiltFigureEight {
                pan_size,
                tilt_size,
                pan_center,
                tilt_center,
            } => match attribute_name {
                "Pan" => Some(
                    (f32::sin(t as f32 * speed * 2.0 - phase_offset_deg.to_radians()))
                        * (pan_size / 2.0)
                        + pan_center,
                ),
                "Tilt" => Some(
                    (f32::sin(t as f32 * speed - phase_offset_deg.to_radians()))
                        * (tilt_size / 2.0)
                        + tilt_center,
                ),
                _ => None,
            },
            _ => None,
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
