use crate::fixture::{handler::FixtureHandler, presets::PresetHandler};

use super::Sequence;

pub struct SequenceRuntime {
    sequence: Sequence,

    current_cue: usize,
    prev_cue: usize,
    started: bool,
}

impl SequenceRuntime {
    pub fn new(sequence: Sequence) -> Self {
        Self {
            prev_cue: sequence.cues().len(),
            sequence,
            current_cue: 0,
            started: false,
        }
    }

    pub fn update(
        &mut self,
        fixture_handler: &mut FixtureHandler,
        _preset_handler: &PresetHandler,
        _delta_time: f64,
    ) {
        if !self.started {
            return;
        }

        if self.current_cue == self.prev_cue {
            return;
        }

        self.prev_cue = self.current_cue;

        let current_cue = self.sequence.cue(self.current_cue);
        for (fixture_id, fixture_channels) in current_cue.data().iter() {
            fixture_handler
                .fixture(*fixture_id)
                .unwrap()
                .update_channels(fixture_channels)
                .expect("");
        }
    }

    pub fn play(&mut self) {
        self.started = true;
    }

    pub fn next_cue(&mut self) {
        self.current_cue += 1;

        if self.current_cue >= self.sequence.cues().len() {
            self.current_cue = 0;
        }
    }
}
