use std::time;

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::{
    channel2::{
        channel_type::FixtureChannelType,
        channel_value::FixtureChannelValue2,
        feature::{feature_config::FixtureFeatureConfig, feature_value::FixtureFeatureValue},
    },
    effect::error::EffectError,
    sequence::FadeFixtureChannelValue,
    value_source::FixtureChannelValuePriority,
};

use super::FeatureEffect;

#[derive(Debug, Serialize, Deserialize, Clone, Default, EguiProbe)]
pub struct FeatureEffectRuntime {
    effect: FeatureEffect,

    #[serde(default)]
    offset: f64,

    #[serde(default, skip_serializing, skip_deserializing)]
    #[egui_probe(skip)]
    effect_started: Option<time::Instant>,
}

impl FeatureEffectRuntime {
    pub fn effect(&self) -> &FeatureEffect {
        &self.effect
    }

    pub fn is_started(&self) -> bool {
        self.effect_started.is_some()
    }

    pub fn start(&mut self) {
        self.effect_started = Some(time::Instant::now());
    }

    pub fn stop(&mut self) {
        self.effect_started = None;
    }

    pub fn get_channel_value(
        &self,
        find_channel_type: FixtureChannelType,
        fixture_feature_configs: &[FixtureFeatureConfig],
        feature_offset_idx: usize,
        priority: FixtureChannelValuePriority,
    ) -> Option<FadeFixtureChannelValue> {
        let channel_types = self
            .effect
            .feature_type()
            .get_channel_types(fixture_feature_configs)
            .ok()?;

        if !channel_types.contains(&find_channel_type) {
            return None;
        }

        let mut channel_values = channel_types
            .into_iter()
            .map(|channel_type| (channel_type, FixtureChannelValue2::Home))
            .collect::<Vec<_>>();

        let feature_value = self.get_feature_value(feature_offset_idx).ok()?;
        feature_value
            .write_back(fixture_feature_configs, &mut channel_values)
            .map_err(EffectError::FixtureChannelError)
            .ok()?;

        channel_values
            .into_iter()
            .find(|(channel_type, _)| *channel_type == find_channel_type)
            .map(|(_, channel_value)| FadeFixtureChannelValue::new(channel_value, 1.0, priority))
    }

    pub fn get_feature_value(
        &self,
        fixture_offset_idx: usize,
    ) -> Result<FixtureFeatureValue, EffectError> {
        self.effect_started
            .ok_or(EffectError::EffectNotStarted)
            .and_then(|effect_started| {
                self.effect.get_feature_value(
                    effect_started.elapsed().as_secs_f64()
                        + (fixture_offset_idx as f64 * self.offset),
                )
            })
    }
}
