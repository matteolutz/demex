use std::time;

use crate::fixture::{channel::FixtureChannel, handler::FixtureHandler, presets::PresetHandler};

use super::Sequence;

pub struct SequenceRuntime {
    sequence: Sequence,

    current_cue: usize,
    cue_update: Option<time::Instant>,
    started: bool,
}

impl SequenceRuntime {
    pub fn new(sequence: Sequence) -> Self {
        Self {
            sequence,
            current_cue: 0,
            cue_update: None,
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

        let delta = self
            .cue_update
            .map(|cu| cu.elapsed().as_secs_f32())
            .unwrap_or(0.0);

        let current_cue = self.sequence.cue(self.current_cue);

        for (fixture_id, fixture_channels) in current_cue.data().iter() {
            let mult = (delta / current_cue.in_fade()).min(1.0);

            let multiplied_channels = fixture_channels
                .iter()
                .map(|c| {
                    if c.snap() {
                        return c.channel().clone();
                    }

                    c.channel().multiply(mult)
                })
                .collect::<Vec<FixtureChannel>>();

            fixture_handler
                .fixture(*fixture_id)
                .unwrap()
                .update_channels(multiplied_channels.iter())
                .expect("");
        }
    }

    pub fn play(&mut self) {
        self.started = true;
        self.current_cue = 0;
        self.cue_update = Some(time::Instant::now());
    }

    pub fn stop(&mut self) {
        self.started = false;
        self.cue_update = None;
    }

    pub fn next_cue(&mut self) {
        self.current_cue += 1;
        self.cue_update = Some(time::Instant::now());

        if self.current_cue >= self.sequence.cues().len() {
            self.current_cue = 0;
        }
    }
}
