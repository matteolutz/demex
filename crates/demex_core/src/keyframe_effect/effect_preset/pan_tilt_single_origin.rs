use serde::{Deserialize, Serialize};

use crate::{
    channel3::{attribute::FixtureChannel3Attribute, clamped_value::ClampedValue},
    keyframe_effect::{
        effect_keyframe::{KeyframeEffectKeyframe, KeyframeEffectKeyframeData},
        effect_keyframe_curve::KeyframeEffectKeyframeCurve,
        effect_layer::KeyframeEffectLayer,
    },
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum PanTiltSingleOriginEffectPresetType {
    Circle,
    Figure8,
    Square,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanTiltSingleOriginEffectPreset {
    /// Origin point for the effect (0.0..=1.0)
    pub origin: [ClampedValue; 2],

    /// Size of the effect (0.0..=1.0)
    pub size: [ClampedValue; 2],

    /// Rotation of the effect (in deg)
    pub rotation: f32,

    /// Move type of the effect
    pub move_type: PanTiltSingleOriginEffectPresetType,
}

impl PanTiltSingleOriginEffectPreset {
    pub fn build_layers(&self) -> Vec<KeyframeEffectLayer> {
        match self.move_type {
            PanTiltSingleOriginEffectPresetType::Circle => {
                let pan_min: ClampedValue = self.origin[0] - (self.size[0] / 2.0);
                let pan_max: ClampedValue = self.origin[0] + (self.size[0] / 2.0);

                let tilt_max: ClampedValue = self.origin[1] + (self.size[1] / 2.0);
                let tilt_min: ClampedValue = self.origin[1] - (self.size[1] / 2.0);

                let layer = KeyframeEffectLayer {
                    keyframes: vec![
                        KeyframeEffectKeyframe {
                            starting_point: 0.0,
                            data: KeyframeEffectKeyframeData::Global(
                                [
                                    (FixtureChannel3Attribute::Pan, pan_min),
                                    (FixtureChannel3Attribute::Tilt, tilt_max),
                                ]
                                .into(),
                            ),
                            curve: KeyframeEffectKeyframeCurve::EaseInOut,
                        },
                        KeyframeEffectKeyframe {
                            starting_point: 0.5,
                            data: KeyframeEffectKeyframeData::Global(
                                [
                                    (FixtureChannel3Attribute::Pan, pan_max),
                                    (FixtureChannel3Attribute::Tilt, tilt_min),
                                ]
                                .into(),
                            ),
                            curve: KeyframeEffectKeyframeCurve::EaseInOut,
                        },
                        KeyframeEffectKeyframe {
                            starting_point: 1.0,
                            data: KeyframeEffectKeyframeData::Global(
                                [
                                    (FixtureChannel3Attribute::Pan, pan_min),
                                    (FixtureChannel3Attribute::Tilt, tilt_max),
                                ]
                                .into(),
                            ),
                            curve: KeyframeEffectKeyframeCurve::EaseInOut,
                        },
                    ],
                    phase_multiplier: 1.0,
                    phase_offset: 0.0,
                };

                vec![layer]
            }
            _ => todo!(),
        }
    }
}
