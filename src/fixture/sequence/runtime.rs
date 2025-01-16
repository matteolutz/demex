use std::time;

use serde::{Deserialize, Serialize};

use crate::fixture::{
    channel::value::{FixtureChannelDiscreteValue, FixtureChannelValue},
    handler::FixtureHandler,
    value_source::FixtureChannelValueSource,
};

use super::{cue::CueTrigger, FadeFixtureChannelValue, Sequence};

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableSequenceRuntime {
    id: u32,
    name: String,
    sequence: Sequence,
}

impl From<&SequenceRuntime> for SerializableSequenceRuntime {
    fn from(value: &SequenceRuntime) -> Self {
        Self {
            id: value.id,
            name: value.name.clone(),
            sequence: value.sequence.clone(),
        }
    }
}

pub struct SequenceRuntime {
    id: u32,
    name: String,
    sequence: Sequence,

    current_cue: usize,
    cue_update: Option<time::Instant>,
    started: bool,
    first_cue: bool,
}

impl From<SerializableSequenceRuntime> for SequenceRuntime {
    fn from(value: SerializableSequenceRuntime) -> Self {
        Self::new(value.id, value.name, value.sequence)
    }
}

impl SequenceRuntime {
    pub fn new(id: u32, name: String, sequence: Sequence) -> Self {
        Self {
            id,
            name,
            sequence,
            current_cue: 0,
            cue_update: None,
            started: false,
            first_cue: true,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_started(&self) -> bool {
        self.started
    }

    pub fn channel_value(
        &self,
        fixture_id: u32,
        channel_id: u16,
    ) -> Option<FadeFixtureChannelValue> {
        if !self.started {
            return None;
        }

        let cue = self.sequence.cue(self.current_cue);
        let prev_cue_idx = self.previous_cue_idx();

        let delta = time::Instant::now()
            .duration_since(self.cue_update.unwrap())
            .as_secs_f32();

        // its the first cue, so we want to fade in from black
        // TODO: this wont work like this
        if self.first_cue {
            let fade = if delta < cue.in_delay() {
                0.0
            } else {
                ((delta - cue.in_delay()) / cue.in_fade()).min(1.0)
            };

            cue.channel_value_for_fixture(fixture_id, channel_id)
                .map(|v| FadeFixtureChannelValue::new(v.clone(), fade))
        } else if prev_cue_idx.is_some() {
            let prev_cue = self.sequence.cue(prev_cue_idx.unwrap());

            let mix = if delta < (prev_cue.out_delay() + cue.in_delay()) {
                0.0
            } else {
                ((delta - (cue.in_delay() + prev_cue.out_delay()))
                    / (cue.in_fade() + prev_cue.out_fade()))
                .min(1.0)
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
                                        .unwrap_or(FixtureChannelValue::Discrete(
                                            FixtureChannelDiscreteValue::AnyHome,
                                        )),
                                ),
                                b: Box::new(v.clone()),
                                mix,
                            },
                            1.0,
                        )
                    });

            if current_cue_value.is_none() {
                prev_cue
                    .channel_value_for_fixture(fixture_id, channel_id)
                    .map(|v| FadeFixtureChannelValue::new(v.clone(), 1.0 - mix))
            } else {
                current_cue_value
            }
        } else {
            None
        }
    }

    pub fn update(&mut self, _delta_time: f64, fixture_handler: &mut FixtureHandler) {
        if !self.started {
            return;
        }

        let delta = time::Instant::now()
            .duration_since(self.cue_update.unwrap())
            .as_secs_f32();

        let previous_cue_idx = self.previous_cue_idx();
        let current_cue = self.sequence.cue(self.current_cue);
        let next_cue_idx = self.next_cue_idx();

        let previous_cue_out_time = previous_cue_idx
            .map(|i| self.sequence.cue(i).out_time())
            .unwrap_or(0.0);

        let cue_time = previous_cue_out_time + current_cue.in_time();

        if delta > cue_time {
            // is the next cue, a follow cue?
            if let Some(next_cue_idx) = next_cue_idx {
                if *self.sequence.cue(next_cue_idx).trigger() == CueTrigger::Follow {
                    self.next_cue();
                }
            } else {
                self.stop(fixture_handler);
            }
        }
    }

    pub fn start(&mut self, fixture_handler: &mut FixtureHandler) {
        self.started = true;
        self.current_cue = 0;
        self.cue_update = Some(time::Instant::now());
        self.first_cue = true;

        for fixture_id in self
            .sequence
            .cues()
            .iter()
            .flat_map(|c| c.data().keys())
            .collect::<Vec<_>>()
            .drain(..)
        {
            if let Some(fixture) = fixture_handler.fixture(*fixture_id) {
                fixture.push_value_source(FixtureChannelValueSource::SequenceRuntime {
                    runtime_id: self.id,
                });
            }
        }
    }

    pub fn stop(&mut self, fixture_handler: &mut FixtureHandler) {
        self.started = false;
        self.cue_update = None;
        self.first_cue = true;

        for fixture_id in self
            .sequence
            .cues()
            .iter()
            .flat_map(|c| c.data().keys())
            .collect::<Vec<_>>()
            .drain(..)
        {
            if let Some(fixture) = fixture_handler.fixture(*fixture_id) {
                fixture.remove_value_source(FixtureChannelValueSource::SequenceRuntime {
                    runtime_id: self.id,
                });
            }
        }
    }

    pub fn should_auto_restart(&self) -> bool {
        return self
            .sequence
            .cues()
            .first()
            .map(|c| *c.trigger() == CueTrigger::Follow)
            .unwrap_or(false);
    }

    pub fn next_cue(&mut self) {
        if self.current_cue == self.sequence.cues().len() - 1 && !self.should_auto_restart() {
            return;
        }

        self.current_cue = (self.current_cue + 1) % self.sequence.cues().len();
        self.cue_update = Some(time::Instant::now());
        self.first_cue = false;
    }

    pub fn previous_cue_idx(&self) -> Option<usize> {
        if self.current_cue == 0 {
            if self.should_auto_restart() {
                Some(self.sequence.cues().len() - 1)
            } else {
                None
            }
        } else {
            Some(self.current_cue - 1)
        }
    }

    pub fn next_cue_idx(&self) -> Option<usize> {
        if self.current_cue == self.sequence.cues().len() - 1 {
            if self.should_auto_restart() {
                Some(0)
            } else {
                None
            }
        } else {
            Some(self.current_cue + 1)
        }
    }
}

impl Serialize for SequenceRuntime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerializableSequenceRuntime::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SequenceRuntime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(SequenceRuntime::from(
            SerializableSequenceRuntime::deserialize(deserializer)?,
        ))
    }
}
