use std::{str::FromStr, time};

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::{
    channel3::{
        channel_value::FixtureChannelValue3, feature::feature_type::FixtureChannel3FeatureType,
    },
    gdtf::GdtfFixture,
    handler::FixtureTypeList,
    presets::PresetHandler,
    timing::TimingHandler,
    value_source::FixtureChannelValuePriority,
};

use super::{cue::CueTrigger, FadeFixtureChannelValue, SequenceStopBehavior};

#[derive(Debug, Clone, Default)]
pub enum SequenceRuntimeState {
    #[default]
    Stopped,

    FirstCue(time::Instant),
    Cue(time::Instant, time::Instant, usize),
}

impl SequenceRuntimeState {
    pub fn is_started(&self) -> bool {
        match self {
            Self::Cue(_, _, _) | Self::FirstCue(_) => true,
            Self::Stopped => false,
        }
    }

    pub fn cue_idx(&self) -> Option<usize> {
        match self {
            Self::FirstCue(_) => Some(0),
            Self::Cue(_, _, idx) => Some(*idx),
            _ => None,
        }
    }

    pub fn when_started(&self) -> Option<(Option<time::Instant>, time::Instant, usize, bool)> {
        match self {
            Self::FirstCue(t) => Some((None, *t, 0, true)),
            Self::Cue(prev_t, t, idx) => Some((Some(*prev_t), *t, *idx, false)),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, EguiProbe)]
pub struct SequenceRuntime {
    sequence_id: u32,

    #[serde(default, skip_serializing, skip_deserializing)]
    #[egui_probe(skip)]
    state: SequenceRuntimeState,
}

impl SequenceRuntime {
    pub fn new(sequence_id: u32) -> Self {
        Self {
            sequence_id,
            state: SequenceRuntimeState::default(),
        }
    }

    pub fn sequence_id(&self) -> u32 {
        self.sequence_id
    }

    pub fn is_started(&self) -> bool {
        self.state.is_started()
    }

    pub fn current_cue(&self) -> Option<usize> {
        self.state.cue_idx()
    }

    pub fn num_cues(&self, preset_handler: &PresetHandler) -> usize {
        preset_handler
            .get_sequence(self.sequence_id)
            .unwrap()
            .cues()
            .len()
    }

    pub fn channel_value(
        &self,
        fixture_types: &FixtureTypeList,
        fixture: &GdtfFixture,
        channel: &gdtf::dmx_mode::DmxChannel,
        speed_multiplier: f32,
        intensity_multiplier: f32,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
        priority: FixtureChannelValuePriority,
    ) -> Option<FadeFixtureChannelValue> {
        if let Some((prev_cue_update, cue_update, cue_idx, is_first_cue)) =
            self.state.when_started()
        {
            let (fixture_type, _) = fixture.fixture_type_and_dmx_mode(fixture_types).ok()?;
            let channel_feature = channel.logical_channels[0]
                .attribute(fixture_type)
                .and_then(|attribute| attribute.feature(&fixture_type.attribute_definitions))
                .and_then(|feature| {
                    FixtureChannel3FeatureType::from_str(feature.name.as_ref().unwrap().as_ref())
                        .ok()
                })?;

            let sequence = preset_handler.get_sequence(self.sequence_id).unwrap();

            if sequence.cues().is_empty() {
                return None;
            }

            let cue = sequence.cue(cue_idx);
            let prev_cue_idx = self.previous_cue_idx(preset_handler);

            let mut delta = time::Instant::now()
                .duration_since(cue_update)
                .as_secs_f32();

            delta *= speed_multiplier;

            delta = f32::max(
                delta - cue.offset_for_fixture(fixture.id(), preset_handler),
                0.0,
            );

            let should_snap =
                cue.should_snap_channel_value_for_fixture(fixture.id(), channel.name().as_ref());

            // its the first cue, so we want to fade in from black
            // TODO: this wont work like this
            if is_first_cue {
                let mut fade = if delta < cue.in_delay() {
                    0.0
                } else {
                    ((delta - cue.in_delay()) / cue.in_fade()).min(1.0)
                };

                if channel_feature == FixtureChannel3FeatureType::Dimmer {
                    fade *= intensity_multiplier;
                }

                if should_snap {
                    fade = if fade >= cue.snap_percent() { 1.0 } else { 0.0 };
                }

                cue.channel_value_for_fixture(
                    fixture,
                    fixture_types,
                    channel.name().as_ref(),
                    preset_handler,
                    timing_handler,
                    Some(cue_update),
                )
                .map(|v| FadeFixtureChannelValue::new(v, fade, priority))
            } else if prev_cue_idx.is_some() {
                // this isn't the first cue, meaning we should fade between the value of the previous cue
                // and the value of the current cue
                let prev_cue = sequence.cue(prev_cue_idx.unwrap());

                let mut mix = if delta < (prev_cue.out_delay() + cue.in_delay()) {
                    0.0
                } else {
                    ((delta - (cue.in_delay() + prev_cue.out_delay()))
                        / (cue.in_fade() + prev_cue.out_fade()))
                    .min(1.0)
                };

                if should_snap {
                    mix = if mix >= cue.snap_percent() { 1.0 } else { 0.0 };
                }

                let fade = if channel_feature == FixtureChannel3FeatureType::Dimmer {
                    intensity_multiplier
                } else {
                    1.0
                };

                let current_cue_value = cue
                    .channel_value_for_fixture(
                        fixture,
                        fixture_types,
                        channel.name().as_ref(),
                        preset_handler,
                        timing_handler,
                        Some(cue_update),
                    )
                    .map(|v| {
                        FadeFixtureChannelValue::new(
                            FixtureChannelValue3::Mix {
                                a: Box::new(
                                    prev_cue
                                        .channel_value_for_fixture(
                                            fixture,
                                            fixture_types,
                                            channel.name().as_ref(),
                                            preset_handler,
                                            timing_handler,
                                            prev_cue_update,
                                        )
                                        .unwrap_or(FixtureChannelValue3::Home),
                                ),
                                b: Box::new(v),
                                mix,
                            },
                            fade,
                            priority,
                        )
                    });

                if current_cue_value.is_none() {
                    prev_cue
                        .channel_value_for_fixture(
                            fixture,
                            fixture_types,
                            channel.name().as_ref(),
                            preset_handler,
                            timing_handler,
                            Some(cue_update),
                        )
                        .map(|v| FadeFixtureChannelValue::new(v, (1.0 - mix) * fade, priority))
                } else {
                    current_cue_value
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn update(&mut self, speed_multiplier: f32, preset_handler: &PresetHandler) -> bool {
        if let Some((_, cue_update, cue_idx, _)) = self.state.when_started() {
            let sequence = preset_handler.get_sequence(self.sequence_id).unwrap();

            if sequence.cues().is_empty() {
                return false;
            }

            let delta = time::Instant::now()
                .duration_since(cue_update)
                .as_secs_f32()
                * speed_multiplier;

            let previous_cue_idx = self.previous_cue_idx(preset_handler);
            let current_cue = sequence.cue(cue_idx);
            let next_cue_idx = self.next_cue_idx(preset_handler);

            let previous_cue_out_time = previous_cue_idx
                .map(|i| sequence.cue(i).out_time(preset_handler))
                .unwrap_or(0.0);

            let cue_time = previous_cue_out_time + current_cue.in_time(preset_handler);

            if delta > cue_time {
                // is the next cue, a follow cue?
                if let Some(next_cue_idx) = next_cue_idx {
                    if *sequence.cue(next_cue_idx).trigger() == CueTrigger::Follow {
                        self.next_cue(preset_handler, 0.0);
                    }
                    // it's the last cue, so we should wait for the out time of the last cue
                    // and then stop the sequence, if the sequence is set to auto stop
                } else if sequence.stop_behavior() == SequenceStopBehavior::AutoStop
                    && delta > cue_time + current_cue.out_time(preset_handler)
                {
                    self.stop();
                    return true;
                }
            }

            false
        } else {
            false
        }
    }

    pub fn start(&mut self, time_offset: f32) {
        self.state = SequenceRuntimeState::FirstCue(
            time::Instant::now() - time::Duration::from_secs_f32(time_offset),
        );
    }

    pub fn stop(&mut self) {
        self.state = SequenceRuntimeState::Stopped
    }

    pub fn should_auto_restart(&self, preset_handler: &PresetHandler) -> bool {
        preset_handler
            .get_sequence(self.sequence_id)
            .unwrap()
            .cues()
            .first()
            .map(|c| *c.trigger() == CueTrigger::Follow)
            .unwrap_or(false)
    }

    pub fn next_cue(&mut self, preset_handler: &PresetHandler, time_offset: f32) -> bool {
        if let Some((_, cue_update, cue_idx, _)) = self.state.when_started() {
            let sequence = preset_handler.get_sequence(self.sequence_id).unwrap();

            if cue_idx == sequence.cues().len() - 1 && !self.should_auto_restart(preset_handler) {
                if sequence.stop_behavior() == SequenceStopBehavior::Restart {
                    self.state = SequenceRuntimeState::Cue(cue_update, time::Instant::now(), 0);
                    return false;
                } else {
                    self.stop();
                    return true;
                }
            }

            let cue_idx = (cue_idx + 1) % sequence.cues().len();
            self.state = SequenceRuntimeState::Cue(
                cue_update,
                time::Instant::now() - time::Duration::from_secs_f32(time_offset),
                cue_idx,
            );
        }

        false
    }

    pub fn previous_cue_idx(&self, preset_handler: &PresetHandler) -> Option<usize> {
        if let Some((_, _, cue_idx, is_first_cue)) = self.state.when_started() {
            let sequence = preset_handler.get_sequence(self.sequence_id).unwrap();

            if cue_idx == 0 {
                // if this is the first cue, we shouldn't return any
                // previous cue. This would distort the fade in time
                // of the first cue
                if !is_first_cue
                    && (self.should_auto_restart(preset_handler)
                        || sequence.stop_behavior() == SequenceStopBehavior::Restart)
                {
                    Some(sequence.cues().len() - 1)
                } else {
                    None
                }
            } else {
                Some(cue_idx - 1)
            }
        } else {
            None
        }
    }

    pub fn next_cue_idx(&self, preset_handler: &PresetHandler) -> Option<usize> {
        if let Some((_, _, cue_idx, _)) = self.state.when_started() {
            let sequence = preset_handler.get_sequence(self.sequence_id).unwrap();

            if cue_idx == sequence.cues().len() - 1 {
                if self.should_auto_restart(preset_handler) {
                    Some(0)
                } else {
                    None
                }
            } else {
                Some(cue_idx + 1)
            }
        } else {
            None
        }
    }
}
