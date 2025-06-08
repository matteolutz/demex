use std::{f32, time};

use serde::{Deserialize, Serialize};

use crate::{
    fixture::{
        channel3::channel_value::FixtureChannelValue3,
        effect::{
            error::EffectError,
            speed::{EffectSpeed, EffectSpeedSyncMode},
        },
        gdtf::GdtfFixture,
        handler::FixtureTypeList,
        keyframe_effect::effect::KeyframeEffect,
        timing::TimingHandler,
        updatables::runtime::RuntimePhase,
    },
    utils::math::instant_diff_secs,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyframeEffectRuntime {
    effect: KeyframeEffect,

    #[serde(default)]
    speed: EffectSpeed,

    #[serde(default)]
    phase: RuntimePhase,
}

impl KeyframeEffectRuntime {
    pub fn effect(&self) -> &KeyframeEffect {
        &self.effect
    }

    pub fn get_channel_value_with_started(
        &self,
        channel_name: &str,
        fixture: &GdtfFixture,
        _fixture_types: &FixtureTypeList,
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

                let channel_value = self.effect.value(
                    fixture.id(),
                    channel_name,
                    started_elapsed,
                    phase_offset,
                    speed_multiplier,
                );

                channel_value.ok_or(EffectError::NoValueForAttribute)
            })
    }
}
