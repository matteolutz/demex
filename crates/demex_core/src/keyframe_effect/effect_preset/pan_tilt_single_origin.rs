use serde::{Deserialize, Serialize};

use crate::{
    channel3::{attribute::FixtureChannel3Attribute, clamped_value::ClampedValue},
    keyframe_effect::{
        effect_keyframe::{KeyframeEffectKeyframe, KeyframeEffectKeyframeData},
        effect_keyframe_curve::KeyframeEffectKeyframeCurve,
        effect_layer::KeyframeEffectLayer,
    },
};

#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    strum::EnumIter,
    strum::Display,
)]
pub enum PanTiltSingleOriginEffectPresetType {
    #[default]
    Ellipse,

    Figure8,
    Rect,
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

impl Default for PanTiltSingleOriginEffectPreset {
    fn default() -> Self {
        Self {
            origin: [0.5.into(), 0.5.into()],
            size: [0.5.into(), 0.5.into()],
            rotation: Default::default(),
            move_type: Default::default(),
        }
    }
}

impl PanTiltSingleOriginEffectPreset {
    pub fn build_layers(&self) -> Vec<KeyframeEffectLayer> {
        match self.move_type {
            PanTiltSingleOriginEffectPresetType::Ellipse => {
                let pan_min: ClampedValue = self.origin[0] - (self.size[0] / 2.0);
                let pan_max: ClampedValue = self.origin[0] + (self.size[0] / 2.0);

                let tilt_max: ClampedValue = self.origin[1] + (self.size[1] / 2.0);
                let tilt_min: ClampedValue = self.origin[1] - (self.size[1] / 2.0);

                let tilt_layer = KeyframeEffectLayer {
                    keyframes: vec![
                        KeyframeEffectKeyframe {
                            starting_point: 0.0,
                            data: KeyframeEffectKeyframeData::Global(
                                [(FixtureChannel3Attribute::Tilt, tilt_max)].into(),
                            ),
                            curve: KeyframeEffectKeyframeCurve::EaseInOut,
                        },
                        KeyframeEffectKeyframe {
                            starting_point: 0.5,
                            data: KeyframeEffectKeyframeData::Global(
                                [(FixtureChannel3Attribute::Tilt, tilt_min)].into(),
                            ),
                            curve: KeyframeEffectKeyframeCurve::EaseInOut,
                        },
                        KeyframeEffectKeyframe {
                            starting_point: 1.0,
                            data: KeyframeEffectKeyframeData::Global(
                                [(FixtureChannel3Attribute::Tilt, tilt_max)].into(),
                            ),
                            curve: KeyframeEffectKeyframeCurve::EaseInOut,
                        },
                    ],
                    phase_multiplier: 1.0,
                    phase_offset: 0.0,
                };

                let pan_layer = KeyframeEffectLayer {
                    keyframes: vec![
                        KeyframeEffectKeyframe {
                            starting_point: 0.0,
                            data: KeyframeEffectKeyframeData::Global(
                                [(FixtureChannel3Attribute::Pan, pan_min)].into(),
                            ),
                            curve: KeyframeEffectKeyframeCurve::EaseInOut,
                        },
                        KeyframeEffectKeyframe {
                            starting_point: 0.5,
                            data: KeyframeEffectKeyframeData::Global(
                                [(FixtureChannel3Attribute::Pan, pan_max)].into(),
                            ),
                            curve: KeyframeEffectKeyframeCurve::EaseInOut,
                        },
                        KeyframeEffectKeyframe {
                            starting_point: 1.0,
                            data: KeyframeEffectKeyframeData::Global(
                                [(FixtureChannel3Attribute::Pan, pan_min)].into(),
                            ),
                            curve: KeyframeEffectKeyframeCurve::EaseInOut,
                        },
                    ],
                    phase_multiplier: 1.0,
                    phase_offset: 90.0,
                };

                vec![tilt_layer, pan_layer]
            }
            PanTiltSingleOriginEffectPresetType::Rect => {
                let pan_min: ClampedValue = self.origin[0] - (self.size[0] / 2.0);
                let pan_max: ClampedValue = self.origin[0] + (self.size[0] / 2.0);

                let tilt_max: ClampedValue = self.origin[1] + (self.size[1] / 2.0);
                let tilt_min: ClampedValue = self.origin[1] - (self.size[1] / 2.0);

                let tilt_layer = KeyframeEffectLayer::new(vec![
                    KeyframeEffectKeyframe {
                        starting_point: 0.0,
                        data: KeyframeEffectKeyframeData::Global(
                            [
                                (FixtureChannel3Attribute::Tilt, tilt_min),
                                (FixtureChannel3Attribute::Pan, pan_min),
                            ]
                            .into(),
                        ),
                        curve: KeyframeEffectKeyframeCurve::Linear,
                    },
                    KeyframeEffectKeyframe {
                        starting_point: 0.25,
                        data: KeyframeEffectKeyframeData::Global(
                            [
                                (FixtureChannel3Attribute::Tilt, tilt_min),
                                (FixtureChannel3Attribute::Pan, pan_max),
                            ]
                            .into(),
                        ),
                        curve: KeyframeEffectKeyframeCurve::Linear,
                    },
                    KeyframeEffectKeyframe {
                        starting_point: 0.5,
                        data: KeyframeEffectKeyframeData::Global(
                            [
                                (FixtureChannel3Attribute::Tilt, tilt_max),
                                (FixtureChannel3Attribute::Pan, pan_max),
                            ]
                            .into(),
                        ),
                        curve: KeyframeEffectKeyframeCurve::Linear,
                    },
                    KeyframeEffectKeyframe {
                        starting_point: 0.75,
                        data: KeyframeEffectKeyframeData::Global(
                            [
                                (FixtureChannel3Attribute::Tilt, tilt_max),
                                (FixtureChannel3Attribute::Pan, pan_min),
                            ]
                            .into(),
                        ),
                        curve: KeyframeEffectKeyframeCurve::Linear,
                    },
                    KeyframeEffectKeyframe {
                        starting_point: 1.0,
                        data: KeyframeEffectKeyframeData::Global(
                            [
                                (FixtureChannel3Attribute::Tilt, tilt_min),
                                (FixtureChannel3Attribute::Pan, pan_min),
                            ]
                            .into(),
                        ),
                        curve: KeyframeEffectKeyframeCurve::Snap,
                    },
                ]);

                vec![tilt_layer]
            }
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f32;

    use crate::{
        channel3::attribute::FixtureChannel3Attribute,
        fpath,
        keyframe_effect::{
            effect_layer::KeyframeEffectLayer,
            effect_preset::{KeyframeEffectPreset, PanTiltSingleOriginEffectPresetType},
        },
    };

    use super::PanTiltSingleOriginEffectPreset;

    fn print_layer(layer: &KeyframeEffectLayer, attribute: &FixtureChannel3Attribute) {
        println!("---------");
        println!("Attribute: {}", attribute);
        let steps = 20;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let phase = t * 2.0 * f32::consts::PI;

            println!(
                "t = {}: {:?}",
                t,
                layer.value(&fpath!(1, 1), attribute, phase)
            );
        }
        println!("---------");
    }

    #[test]
    fn test_rect_a() {
        let preset = KeyframeEffectPreset::PanTiltSingleOrigin(PanTiltSingleOriginEffectPreset {
            origin: [0.5.into(), 0.5.into()],
            size: [1.0.into(), 1.0.into()],
            rotation: 0.0,
            move_type: PanTiltSingleOriginEffectPresetType::Rect,
        });

        let layers = preset.build_layers();
        let layer = &layers[0];

        print_layer(layer, &FixtureChannel3Attribute::Pan);
        print_layer(layer, &FixtureChannel3Attribute::Tilt);
    }

    #[test]
    fn test_ellipse_a() {
        let preset = KeyframeEffectPreset::PanTiltSingleOrigin(PanTiltSingleOriginEffectPreset {
            origin: [0.5.into(), 0.5.into()],
            size: [1.0.into(), 1.0.into()],
            rotation: 0.0,
            move_type: PanTiltSingleOriginEffectPresetType::Ellipse,
        });

        let layers = preset.build_layers();
        let tilt_layer = &layers[0];
        let pan_layer = &layers[1];

        print_layer(pan_layer, &FixtureChannel3Attribute::Pan);
        print_layer(tilt_layer, &FixtureChannel3Attribute::Tilt);
    }
}
