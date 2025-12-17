use std::{f32, time};

use serde::{Deserialize, Serialize};

use crate::{
    channel3::{
        attribute::FixtureChannel3Attribute, channel_value::FixtureChannelValue3,
        channel_value_discrete::FixtureChannelDiscreteValue,
    },
    effect::{
        error::EffectError,
        speed::{EffectSpeed, EffectSpeedSyncMode},
    },
    effect2::effect::Effect2,
    fixture::Fixture,
    timing::TimingHandler,
    updatables::runtime::RuntimePhase,
    utils::math::instant_diff_secs,
};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct FeatureEffectRuntime {
    // effect: FeatureEffect,
    effect: Effect2,

    #[serde(default)]
    speed: EffectSpeed,

    #[serde(default)]
    phase: RuntimePhase,

    #[serde(default, skip_serializing, skip_deserializing)]
    effect_started: Option<time::Instant>,
}

impl FeatureEffectRuntime {
    pub fn new(effect: Effect2) -> Self {
        Self {
            effect,
            ..Default::default()
        }
    }

    pub fn effect(&self) -> &Effect2 {
        &self.effect
    }

    pub fn effect_mut(&mut self) -> &mut Effect2 {
        &mut self.effect
    }

    pub fn phase_mut(&mut self) -> &mut RuntimePhase {
        &mut self.phase
    }

    pub fn speed_mut(&mut self) -> &mut EffectSpeed {
        &mut self.speed
    }

    pub fn is_started(&self) -> bool {
        self.effect_started.is_some()
    }

    pub fn start(&mut self, time_offset: f32) {
        self.effect_started =
            Some(time::Instant::now() - time::Duration::from_secs_f32(time_offset));
    }

    pub fn stop(&mut self) {
        self.effect_started = None;
    }

    pub fn get_values_with_started(
        &self,
        fixture: &Fixture,
        fixture_offset: f32,
        timing_handler: &TimingHandler,
        started: Option<time::Instant>,
    ) -> Vec<(FixtureChannel3Attribute, FixtureChannelValue3)> {
        self.effect()
            .attributes()
            .filter(|attribute| fixture.has_attribute(attribute))
            .filter_map(|attribute| {
                self.get_channel_value_with_started(
                    attribute,
                    fixture_offset,
                    timing_handler,
                    started,
                )
                .ok()
                .map(|value| (*attribute, value))
            })
            .collect::<Vec<_>>()
    }

    pub fn get_channel_value(
        &self,
        attribute: &FixtureChannel3Attribute,
        fixture_offset: f32,
        timing_handler: &TimingHandler,
    ) -> Result<FixtureChannelValue3, EffectError> {
        self.get_channel_value_with_started(
            attribute,
            fixture_offset,
            timing_handler,
            self.effect_started,
        )
    }

    pub fn get_channel_value_with_started(
        &self,
        attribute: &FixtureChannel3Attribute,
        fixture_offset: f32,
        timing_handler: &TimingHandler,
        started: Option<time::Instant>,
    ) -> Result<FixtureChannelValue3, EffectError> {
        started
            .ok_or(EffectError::EffectNotStarted)
            .and_then(|effect_started| {
                let phase_offset = self.phase.phase(fixture_offset);
                let mut started_elapsed = effect_started.elapsed().as_secs_f64();

                let effective_bpm = match &self.speed {
                    EffectSpeed::SpeedMaster { id, scale, sync } => {
                        if let Ok(speed_master_value) = timing_handler.get_speed_master_value(*id) {
                            if sync.is_synced() {
                                if let Some(interval) = speed_master_value.interval() {
                                    let mut mod_value = speed_master_value.secs_per_beat();
                                    if *sync == EffectSpeedSyncMode::SyncBeat {
                                        mod_value *= 1.0 / scale.scale_value();
                                    }

                                    let delta = instant_diff_secs(effect_started, interval)
                                        % mod_value as f64;

                                    started_elapsed += delta;
                                }
                            }

                            speed_master_value.bpm() * scale.scale_value()
                        } else {
                            0.0
                        }
                    }
                    EffectSpeed::Bpm(bpm) => *bpm,
                };

                let effective_bps = effective_bpm / 60.0;
                let speed_multiplier = (2.0 * f32::consts::PI) * effective_bps;

                let attribute_value = self.effect.attribute_value(
                    attribute,
                    started_elapsed,
                    phase_offset,
                    speed_multiplier,
                );

                attribute_value
                    .map(|val| {
                        FixtureChannelValue3::Discrete(FixtureChannelDiscreteValue::discrete(val))
                    })
                    .ok_or(EffectError::NoValueForAttribute)
            })
    }

    /*
    pub fn get_channel_value(
        &self,
        find_channel_type: FixtureChannelType,
        fixture_feature_configs: &[FixtureFeatureConfig],
        fixture_offset: f32,
        priority: FixtureChannelValuePriority,
        timing_handler: &TimingHandler,
    ) -> Option<FadeFixtureChannelValue> {
        self.get_channel_value_with_started(
            find_channel_type,
            fixture_feature_configs,
            fixture_offset,
            priority,
            timing_handler,
            self.effect_started,
        )
    }

    pub fn get_channel_value_with_started(
        &self,
        find_channel_type: FixtureChannelType,
        fixture_feature_configs: &[FixtureFeatureConfig],
        fixture_offset: f32,
        priority: FixtureChannelValuePriority,
        timing_handler: &TimingHandler,
        started: Option<time::Instant>,
    ) -> Option<FadeFixtureChannelValue> {
        todo!();
        /*
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

        let feature_value = self
            .get_feature_value_with_started(fixture_offset, timing_handler, started)
            .ok()?;
        feature_value
            .write_back(fixture_feature_configs, &mut channel_values)
            .map_err(EffectError::FixtureChannelError)
            .ok()?;

        channel_values
            .into_iter()
            .find(|(channel_type, _)| *channel_type == find_channel_type)
            .map(|(_, channel_value)| FadeFixtureChannelValue::new(channel_value, 1.0, priority))
            */
    }

    pub fn get_feature_value(
        &self,
        fixture_offset: f32,
        timing_handler: &TimingHandler,
    ) -> Result<FixtureFeatureValue, EffectError> {
        self.get_feature_value_with_started(fixture_offset, timing_handler, self.effect_started)
    }

    pub fn get_feature_value_with_started(
        &self,
        fixture_offset: f32,
        timing_handler: &TimingHandler,
        started: Option<time::Instant>,
    ) -> Result<FixtureFeatureValue, EffectError> {
        started
            .ok_or(EffectError::EffectNotStarted)
            .and_then(|effect_started| {
                let phase_offset = self.phase.phase(fixture_offset);
                let mut started_elapsed = effect_started.elapsed().as_secs_f64();

                let effective_bpm = match &self.speed {
                    EffectSpeed::SpeedMaster { id, scale, sync } => {
                        if let Ok(speed_master_value) = timing_handler.get_speed_master_value(*id) {
                            if sync.is_synced() {
                                if let Some(interval) = speed_master_value.interval() {
                                    let mut mod_value = speed_master_value.secs_per_beat();
                                    if *sync == EffectSpeedSyncMode::SyncBeat {
                                        mod_value *= 1.0 / scale.scale_value();
                                    }

                                    let delta = instant_diff_secs(effect_started, interval)
                                        % mod_value as f64;

                                    started_elapsed += delta;
                                }
                            }

                            speed_master_value.bpm() * scale.scale_value()
                        } else {
                            0.0
                        }
                    }
                    EffectSpeed::Bpm(bpm) => *bpm,
                };

                let effective_bps = effective_bpm / 60.0;
                let speed_multiplier = (2.0 * f32::consts::PI) * effective_bps;

                self.effect
                    .get_feature_value(started_elapsed, phase_offset, speed_multiplier)
            })
    }
    */
}
