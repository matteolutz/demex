use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::fixture::{
    channel3::channel_value::FixtureChannelValue3,
    keyframe_effect::effect_keyframe_curve::KeyframeEffectKeyframeCurve,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyframeEffectKeyframe {
    starting_point: f32,
    values: HashMap<u32, HashMap<String, FixtureChannelValue3>>,
    curve: KeyframeEffectKeyframeCurve,
}

impl KeyframeEffectKeyframe {
    pub fn new(
        starting_point: f32,
        values: HashMap<u32, HashMap<String, FixtureChannelValue3>>,
        curve: KeyframeEffectKeyframeCurve,
    ) -> Self {
        Self {
            starting_point,
            values,
            curve,
        }
    }

    pub fn affected_fixtures(&self) -> Vec<u32> {
        self.values.keys().cloned().collect()
    }

    pub fn affected_channels_for_fixture(&self, fixture_id: u32) -> Option<Vec<&str>> {
        self.values
            .get(&fixture_id)
            .map(|channels| channels.keys().map(String::as_str).collect())
    }

    pub fn absolute_starting_point(&self, num_keyframes: usize, idx: usize) -> f32 {
        let default_keyframe_duration = 1.0 / num_keyframes as f32;
        (self.starting_point / num_keyframes as f32) + idx as f32 * default_keyframe_duration
    }

    pub fn value(&self, fixture_id: u32, channel: &str) -> Option<&FixtureChannelValue3> {
        self.values
            .get(&fixture_id)
            .and_then(|channels| channels.get(channel))
    }

    pub fn value_at(
        &self,
        fixture_id: u32,
        channel: &str,
        t: f32,
    ) -> Option<(&FixtureChannelValue3, f32)> {
        // t is in the range 0.0..=1.0

        self.values
            .get(&fixture_id)
            .and_then(|channels| channels.get(channel))
            .map(|channel_value| (channel_value, self.curve.value(t)))
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn test_keyframe_absolute_starting_point_a() {
        let keyframe = KeyframeEffectKeyframe {
            starting_point: 0.0,
            ..Default::default()
        };

        assert_relative_eq!(
            keyframe.absolute_starting_point(10, 0),
            0.0,
            epsilon = f32::EPSILON
        );
        assert_relative_eq!(
            keyframe.absolute_starting_point(10, 1),
            0.1,
            epsilon = f32::EPSILON
        );
        assert_relative_eq!(
            keyframe.absolute_starting_point(10, 9),
            0.9,
            epsilon = f32::EPSILON
        );
    }

    #[test]
    fn test_keyframe_absolute_starting_point_b() {
        let keyframe = KeyframeEffectKeyframe {
            starting_point: 0.5,
            ..Default::default()
        };

        assert_relative_eq!(
            keyframe.absolute_starting_point(10, 0),
            0.05,
            epsilon = f32::EPSILON
        );
        assert_relative_eq!(
            keyframe.absolute_starting_point(10, 1),
            0.15,
            epsilon = f32::EPSILON
        );
        assert_relative_eq!(
            keyframe.absolute_starting_point(10, 9),
            0.95,
            epsilon = f32::EPSILON
        );
    }

    #[test]
    fn test_keyframe_absolute_starting_point_c() {
        let keyframe = KeyframeEffectKeyframe {
            starting_point: 1.0,
            ..Default::default()
        };

        assert_relative_eq!(
            keyframe.absolute_starting_point(10, 0),
            0.1,
            epsilon = f32::EPSILON
        );
        assert_relative_eq!(
            keyframe.absolute_starting_point(10, 1),
            0.2,
            epsilon = f32::EPSILON
        );
        assert_relative_eq!(
            keyframe.absolute_starting_point(10, 9),
            1.0,
            epsilon = f32::EPSILON
        );
    }
}
