use std::time;

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::{
    channel::{
        value::FixtureChannelValue, value_source::FixtureChannelValuePriority,
        FIXTURE_CHANNEL_INTENSITY_ID,
    },
    presets::PresetHandler,
};

use super::{cue::CueTrigger, FadeFixtureChannelValue};

#[derive(Debug, Serialize, Deserialize, Clone, Default, EguiProbe)]
pub struct SequenceRuntime {
    sequence_id: u32,

    #[serde(default, skip_serializing, skip_deserializing)]
    #[egui_probe(skip)]
    current_cue: usize,

    #[serde(default, skip_serializing, skip_deserializing)]
    #[egui_probe(skip)]
    cue_update: Option<time::Instant>,

    #[serde(default, skip_serializing, skip_deserializing)]
    #[egui_probe(skip)]
    started: bool,

    #[serde(default, skip_serializing, skip_deserializing)]
    #[egui_probe(skip)]
    first_cue: bool,
}

impl SequenceRuntime {
    pub fn new(sequence_id: u32) -> Self {
        Self {
            sequence_id,
            current_cue: 0,
            cue_update: None,
            started: false,
            first_cue: true,
        }
    }

    pub fn sequence_id(&self) -> u32 {
        self.sequence_id
    }

    pub fn is_started(&self) -> bool {
        self.started
    }

    pub fn current_cue(&self) -> usize {
        self.current_cue
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
        fixture_id: u32,
        channel_id: u16,
        speed_multiplier: f32,
        intensity_multiplier: f32,
        preset_handler: &PresetHandler,
        priority: FixtureChannelValuePriority,
    ) -> Option<FadeFixtureChannelValue> {
        if !self.started {
            return None;
        }

        let sequence = preset_handler.get_sequence(self.sequence_id).unwrap();

        if sequence.cues().is_empty() {
            return None;
        }

        let cue = sequence.cue(self.current_cue);
        let prev_cue_idx = self.previous_cue_idx(preset_handler);

        let mut delta = time::Instant::now()
            .duration_since(self.cue_update.unwrap())
            .as_secs_f32();

        delta *= speed_multiplier;

        delta = f32::max(delta - cue.offset_for_fixture(fixture_id), 0.0);

        let should_snap = cue.should_snap_channel_value_for_fixture(fixture_id, channel_id);

        // its the first cue, so we want to fade in from black
        // TODO: this wont work like this
        if self.first_cue {
            let mut fade = if delta < cue.in_delay() {
                0.0
            } else {
                ((delta - cue.in_delay()) / cue.in_fade()).min(1.0)
            };

            if should_snap {
                fade = if fade >= cue.snap_percent() { 1.0 } else { 0.0 };
            }

            if channel_id == FIXTURE_CHANNEL_INTENSITY_ID {
                fade *= intensity_multiplier;
            }

            cue.channel_value_for_fixture(fixture_id, channel_id)
                .map(|v| FadeFixtureChannelValue::new(v.clone(), fade, priority))
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

            let fade = if channel_id == FIXTURE_CHANNEL_INTENSITY_ID {
                intensity_multiplier
            } else {
                1.0
            };

            let current_cue_value =
                cue.channel_value_for_fixture(fixture_id, channel_id)
                    .map(|v| {
                        FadeFixtureChannelValue::new(
                            FixtureChannelValue::Mix {
                                a: Box::new(
                                    prev_cue
                                        .channel_value_for_fixture(fixture_id, channel_id)
                                        .cloned()
                                        .unwrap_or(FixtureChannelValue::any_home()),
                                ),
                                b: Box::new(v.clone()),
                                mix,
                            },
                            fade,
                            priority,
                        )
                    });

            if current_cue_value.is_none() {
                prev_cue
                    .channel_value_for_fixture(fixture_id, channel_id)
                    .map(|v| FadeFixtureChannelValue::new(v.clone(), (1.0 - mix) * fade, priority))
            } else {
                current_cue_value
            }
        } else {
            None
        }
    }

    pub fn update(
        &mut self,
        _delta_time: f64,
        speed_multiplier: f32,
        preset_handler: &PresetHandler,
    ) -> bool {
        if !self.started {
            return false;
        }

        let sequence = preset_handler.get_sequence(self.sequence_id).unwrap();

        if sequence.cues().is_empty() {
            return false;
        }

        let delta = time::Instant::now()
            .duration_since(self.cue_update.unwrap())
            .as_secs_f32()
            * speed_multiplier;

        let previous_cue_idx = self.previous_cue_idx(preset_handler);
        let current_cue = sequence.cue(self.current_cue);
        let next_cue_idx = self.next_cue_idx(preset_handler);

        let previous_cue_out_time = previous_cue_idx
            .map(|i| sequence.cue(i).out_time())
            .unwrap_or(0.0);

        let cue_time = previous_cue_out_time + current_cue.in_time();

        if delta > cue_time {
            // is the next cue, a follow cue?
            if let Some(next_cue_idx) = next_cue_idx {
                if *sequence.cue(next_cue_idx).trigger() == CueTrigger::Follow {
                    self.next_cue(preset_handler);
                }
                // it's the last cue, so we should wait for the out time of the last cue
                // and then stop the sequence
            } /* else if delta > cue_time + current_cue.out_time() {
                  self.stop();
                  return true;
              }*/
        }

        false
    }

    pub fn start(&mut self) {
        self.started = true;
        self.current_cue = 0;
        self.cue_update = Some(time::Instant::now());
        self.first_cue = true;
    }

    pub fn stop(&mut self) {
        self.started = false;
        self.cue_update = None;
        self.first_cue = true;
    }

    pub fn should_auto_restart(&self, preset_handler: &PresetHandler) -> bool {
        return preset_handler
            .get_sequence(self.sequence_id)
            .unwrap()
            .cues()
            .first()
            .map(|c| *c.trigger() == CueTrigger::Follow)
            .unwrap_or(false);
    }

    pub fn next_cue(&mut self, preset_handler: &PresetHandler) {
        let sequence = preset_handler.get_sequence(self.sequence_id).unwrap();

        if self.current_cue == sequence.cues().len() - 1
            && !self.should_auto_restart(preset_handler)
        {
            self.stop();
            return;
        }

        self.current_cue = (self.current_cue + 1) % sequence.cues().len();
        self.cue_update = Some(time::Instant::now());
        self.first_cue = false;
    }

    pub fn previous_cue_idx(&self, preset_handler: &PresetHandler) -> Option<usize> {
        let sequence = preset_handler.get_sequence(self.sequence_id).unwrap();

        if self.current_cue == 0 {
            if self.should_auto_restart(preset_handler) {
                Some(sequence.cues().len() - 1)
            } else {
                None
            }
        } else {
            Some(self.current_cue - 1)
        }
    }

    pub fn next_cue_idx(&self, preset_handler: &PresetHandler) -> Option<usize> {
        let sequence = preset_handler.get_sequence(self.sequence_id).unwrap();

        if self.current_cue == sequence.cues().len() - 1 {
            if self.should_auto_restart(preset_handler) {
                Some(0)
            } else {
                None
            }
        } else {
            Some(self.current_cue + 1)
        }
    }
}
