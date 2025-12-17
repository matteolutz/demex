use std::collections::BTreeSet;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    channel3::{attribute::FixtureChannel3Attribute, channel_value::FixtureChannelValue3},
    fixture::FixturePath,
    keyframe_effect::effect_keyframe::KeyframeEffectKeyframe,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyframeEffectLayer {
    keyframes: Vec<KeyframeEffectKeyframe>,
}

impl KeyframeEffectLayer {
    pub fn new(keyframes: Vec<KeyframeEffectKeyframe>) -> Self {
        Self { keyframes }
    }

    pub fn add_keyframe(&mut self, keyframe: KeyframeEffectKeyframe) {
        self.keyframes.push(keyframe);
    }

    pub fn affected_fixtures(&self) -> BTreeSet<FixturePath> {
        self.keyframes
            .iter()
            .flat_map(|kf| kf.affected_fixtures())
            .collect()
    }

    pub fn affected_attributes_for_fixture(
        &self,
        fixture_path: &FixturePath,
    ) -> Vec<FixtureChannel3Attribute> {
        self.keyframes
            .iter()
            .flat_map(|kf| kf.affected_attributes_for_fixture(fixture_path))
            .flatten()
            .collect()
    }

    pub fn value(
        &self,
        fixture_path: &FixturePath,
        attribute: &FixtureChannel3Attribute,
        t: f32,
    ) -> Option<FixtureChannelValue3> {
        let keyframe = self
            .keyframes
            .iter()
            .enumerate()
            .tuple_windows()
            .find_map(|((idx, kf), (next_idx, next_kf))| {
                if next_kf.absolute_starting_point(self.keyframes.len(), next_idx) > t {
                    Some((idx, kf))
                } else {
                    None
                }
            })
            .or_else(|| {
                self.keyframes
                    .last()
                    .map(|last| (self.keyframes.len() - 1, last))
            });

        if let Some((keyframe_idx, keyframe)) = keyframe {
            let keyframe_starting_point =
                keyframe.absolute_starting_point(self.keyframes.len(), keyframe_idx);

            let keyframe_end_point = if keyframe_idx + 1 < self.keyframes.len() {
                self.keyframes[keyframe_idx + 1]
                    .absolute_starting_point(self.keyframes.len(), keyframe_idx + 1)
            } else {
                1.0
            };

            // map t to the range of keyframe_starting_point to keyframe_end_point
            let keyframe_t =
                (t - keyframe_starting_point) / (keyframe_end_point - keyframe_starting_point);

            keyframe
                .value_at(fixture_path, attribute, keyframe_t)
                .and_then(|(value, fade)| {
                    if fade == 0.0 {
                        Some(value.clone())
                    } else {
                        let next_value =
                            self.keyframes[if keyframe_idx < self.keyframes.len() - 1 {
                                keyframe_idx + 1
                            } else {
                                0
                            }]
                            .value(fixture_path, attribute);

                        next_value.map(|next_value| FixtureChannelValue3::Mix {
                            a: Box::new(value.clone()),
                            b: Box::new(next_value.clone()),
                            mix: fade,
                        })
                    }
                })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        channel3::channel_value_discrete::FixtureChannelDiscreteValue, fixture::FixtureId,
        keyframe_effect::effect_keyframe_curve::KeyframeEffectKeyframeCurve,
    };

    use super::*;

    const TEST_ATTRIBUTE: FixtureChannel3Attribute = FixtureChannel3Attribute::Dimmer;
    fn test_fixture_path() -> FixturePath {
        FixtureId::new(1).unwrap().into()
    }

    fn get_test_values(
        value: FixtureChannelValue3,
    ) -> HashMap<FixturePath, HashMap<FixtureChannel3Attribute, FixtureChannelValue3>> {
        let mut values = HashMap::new();
        let mut channels = HashMap::new();
        channels.insert(TEST_ATTRIBUTE, value);
        values.insert(test_fixture_path(), channels);
        values
    }

    #[test]
    fn test_keyframe_effect_layer_basic_a() {
        let layer = KeyframeEffectLayer {
            keyframes: vec![
                KeyframeEffectKeyframe::new(
                    0.0,
                    get_test_values(FixtureChannelValue3::Discrete(
                        FixtureChannelDiscreteValue::discrete(0.0),
                    )),
                    KeyframeEffectKeyframeCurve::Linear,
                ),
                KeyframeEffectKeyframe::new(
                    0.5,
                    get_test_values(FixtureChannelValue3::Discrete(
                        FixtureChannelDiscreteValue::discrete(1.0),
                    )),
                    KeyframeEffectKeyframeCurve::Linear,
                ),
            ],
        };

        let steps = 10;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            println!(
                "t = {}: {:?}",
                t,
                layer.value(&test_fixture_path(), &TEST_ATTRIBUTE, t)
            );
        }
    }
}
