use std::f32;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::fixture::{
    channel3::channel_value::FixtureChannelValue3,
    keyframe_effect::effect_layer::KeyframeEffectLayer,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyframeEffect {
    layers: Vec<KeyframeEffectLayer>,
}

impl KeyframeEffect {
    pub fn affected_fixtures(&self) -> Vec<u32> {
        self.layers
            .iter()
            .flat_map(|layer| layer.affected_fixtures())
            .dedup()
            .collect()
    }

    pub fn affected_channels_for_fixture(&self, fixture_id: u32) -> Vec<&str> {
        self.layers
            .iter()
            .flat_map(|layer| layer.affected_channels_for_fixture(fixture_id))
            .dedup()
            .collect()
    }

    pub fn value(
        &self,
        fixture_id: u32,
        channel: &str,
        started_elapsed: f64,
        phase_offset_deg: f32,
        speed_multiplier: f32,
    ) -> Option<FixtureChannelValue3> {
        let time_adjusted =
            (started_elapsed as f32 * speed_multiplier) - phase_offset_deg.to_radians();

        // convert time_adjusted to a value between 0.0 and 1.0 (from 0.0 to 2π)
        let t = (time_adjusted % (2.0 * f32::consts::PI)) / (2.0 * f32::consts::PI);

        self.layers
            .iter()
            .flat_map(|layer| layer.value(fixture_id, channel, t))
            .next()
    }
}
