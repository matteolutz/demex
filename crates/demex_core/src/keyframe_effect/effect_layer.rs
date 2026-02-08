use std::collections::HashSet;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::f32;

use crate::{
    channel3::{attribute::FixtureChannel3Attribute, clamped_value::ClampedValue},
    fixture::FixturePath,
    keyframe_effect::effect_keyframe::KeyframeEffectKeyframe,
};

fn f32_one() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyframeEffectLayer {
    pub(crate) keyframes: Vec<KeyframeEffectKeyframe>,

    /// Phase offset for this layer (in deg)
    #[serde(default)]
    pub(crate) phase_offset: f32,

    /// Phase multiplier for this layer (has to >= 1.0)
    #[serde(default = "f32_one")]
    pub(crate) phase_multiplier: f32,
}

impl KeyframeEffectLayer {
    pub fn new(keyframes: Vec<KeyframeEffectKeyframe>) -> Self {
        Self {
            keyframes,
            phase_offset: 0.0,
            phase_multiplier: 1.0,
        }
    }

    pub fn phase_multiplier(&self) -> f32 {
        self.phase_multiplier
    }

    pub fn phase_multiplier_mut(&mut self) -> &mut f32 {
        &mut self.phase_multiplier
    }

    pub fn add_keyframe(&mut self, mut keyframe: KeyframeEffectKeyframe) {
        // move all keyframes over to the left
        let move_by = 1.0 / self.keyframes.len() as f32;
        for keyframe in &mut self.keyframes {
            keyframe.starting_point = (keyframe.starting_point - move_by).max(0.0);
        }

        // move the new keyframe to the end
        keyframe.starting_point = 1.0;

        self.keyframes.push(keyframe);
    }

    pub fn is_affected(&self, fixture_path: &FixturePath) -> bool {
        self.keyframes.iter().any(|kf| kf.is_affected(fixture_path))
    }

    pub fn attributes(&self) -> HashSet<FixtureChannel3Attribute> {
        self.keyframes
            .iter()
            .flat_map(|keyframe| keyframe.attributes())
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

    pub fn keyframes(&self) -> &[KeyframeEffectKeyframe] {
        &self.keyframes
    }

    pub fn keyframes_mut(&mut self) -> &mut Vec<KeyframeEffectKeyframe> {
        &mut self.keyframes
    }

    pub fn value(
        &self,
        fixture_path: &FixturePath,
        attribute: &FixtureChannel3Attribute,
        time_adjusted: f32,
    ) -> Option<ClampedValue> {
        let t = (time_adjusted - self.phase_offset.to_radians()) * self.phase_multiplier;

        let t = (t % (2.0 * f32::consts::PI)) / (2.0 * f32::consts::PI);

        let (keyframe_idx, keyframe) = self
            .keyframes
            .iter()
            .enumerate()
            .tuple_windows()
            .find_map(|((idx, kf), (_, next_kf))| {
                if next_kf.starting_point > t {
                    Some((idx, kf))
                } else {
                    None
                }
            })
            .or_else(|| {
                self.keyframes
                    .last()
                    .map(|last| (self.keyframes.len() - 1, last))
            })?;

        let keyframe_starting_point = keyframe.starting_point;

        let keyframe_end_point = if keyframe_idx + 1 < self.keyframes.len() {
            self.keyframes[keyframe_idx + 1].starting_point
        } else {
            1.0
        };

        // map t to the range of keyframe_starting_point to keyframe_end_point
        let keyframe_t =
            (t - keyframe_starting_point) / (keyframe_end_point - keyframe_starting_point);

        let is_last_keyframe = keyframe_idx == self.keyframes.len() - 1;

        keyframe
            .value_at(fixture_path, attribute, keyframe_t)
            .and_then(|(value, fade)| {
                if is_last_keyframe || fade == 0.0 {
                    Some(value.clone())
                } else {
                    let next_value =
                        self.keyframes[keyframe_idx + 1].value(fixture_path, attribute);

                    // mix between value and next_value based on fade
                    next_value.map(|next_value| (*next_value * fade) + (*value * (1.0 - fade)))
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        fixture::FixtureId, keyframe_effect::effect_keyframe_curve::KeyframeEffectKeyframeCurve,
    };

    use super::*;

    const TEST_ATTRIBUTE: FixtureChannel3Attribute = FixtureChannel3Attribute::Dimmer;
    fn test_fixture_path() -> FixturePath {
        FixtureId::new(1).unwrap().into()
    }

    fn get_test_values(
        value: ClampedValue,
    ) -> HashMap<FixturePath, HashMap<FixtureChannel3Attribute, ClampedValue>> {
        let mut values = HashMap::new();
        let mut channels = HashMap::new();
        channels.insert(TEST_ATTRIBUTE, value);
        values.insert(test_fixture_path(), channels);
        values
    }

    fn print_layer(layer: &KeyframeEffectLayer) {
        println!("---------");
        let steps = 20;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            println!(
                "t = {}: {:?}",
                t,
                layer.value(&test_fixture_path(), &TEST_ATTRIBUTE, t)
            );
        }
        println!("---------");
    }

    #[test]
    fn test_keyframe_effect_layer_basic_a() {
        let mut layer = KeyframeEffectLayer::new(vec![
            KeyframeEffectKeyframe::new(
                0.0,
                get_test_values(0.0.try_into().unwrap()),
                KeyframeEffectKeyframeCurve::Linear,
            ),
            KeyframeEffectKeyframe::new(
                1.0,
                get_test_values(1.0.try_into().unwrap()),
                KeyframeEffectKeyframeCurve::Linear,
            ),
        ]);

        print_layer(&layer);

        layer.add_keyframe(KeyframeEffectKeyframe::new(
            0.0,
            get_test_values(0.0.try_into().unwrap()),
            KeyframeEffectKeyframeCurve::Linear,
        ));

        print_layer(&layer);

        layer.add_keyframe(KeyframeEffectKeyframe::new(
            0.0,
            get_test_values(1.0.try_into().unwrap()),
            KeyframeEffectKeyframeCurve::Linear,
        ));

        print_layer(&layer);
    }
}
