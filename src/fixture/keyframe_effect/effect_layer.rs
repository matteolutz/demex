use std::collections::BTreeSet;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::fixture::{
    channel3::channel_value::FixtureChannelValue3,
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

    pub fn affected_fixtures(&self) -> BTreeSet<u32> {
        self.keyframes
            .iter()
            .flat_map(|kf| kf.affected_fixtures())
            .collect()
    }

    pub fn affected_channels_for_fixture(&self, fixture_id: u32) -> Vec<&str> {
        self.keyframes
            .iter()
            .flat_map(|kf| kf.affected_channels_for_fixture(fixture_id))
            .flatten()
            .collect()
    }

    pub fn value(&self, fixture_id: u32, channel: &str, t: f32) -> Option<FixtureChannelValue3> {
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
                .value_at(fixture_id, channel, keyframe_t)
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
                            .value(fixture_id, channel);

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

    use crate::fixture::keyframe_effect::effect_keyframe_curve::KeyframeEffectKeyframeCurve;

    use super::*;

    const TEST_FIXTURE_ID: u32 = 1;
    const TEST_CHANNEL_NAME: &str = "Test_Dimmer";

    fn get_test_values(
        value: FixtureChannelValue3,
    ) -> HashMap<u32, HashMap<String, FixtureChannelValue3>> {
        let mut values = HashMap::new();
        let mut channels = HashMap::new();
        channels.insert(TEST_CHANNEL_NAME.to_string(), value);
        values.insert(TEST_FIXTURE_ID, channels);
        values
    }

    #[test]
    fn test_keyframe_effect_layer_basic_a() {
        let layer = KeyframeEffectLayer {
            keyframes: vec![
                KeyframeEffectKeyframe::new(
                    0.0,
                    get_test_values(FixtureChannelValue3::Discrete {
                        channel_function_idx: 0,
                        value: 0.0,
                    }),
                    KeyframeEffectKeyframeCurve::Linear,
                ),
                KeyframeEffectKeyframe::new(
                    0.5,
                    get_test_values(FixtureChannelValue3::Discrete {
                        channel_function_idx: 0,
                        value: 1.0,
                    }),
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
                layer.value(TEST_FIXTURE_ID, TEST_CHANNEL_NAME, t)
            );
        }
    }
}
