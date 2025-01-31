use serde::{Deserialize, Serialize};

use crate::fixture::{handler::FixtureHandler, value_source::FixtureChannelValueSource};

use super::{runtime::SequenceRuntime, FadeFixtureChannelValue, Sequence};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceRuntimePreset {
    id: u32,
    name: String,
    /*sequence: Sequence,

    current_cue: usize,
    cue_update: Option<time::Instant>,
    started: bool,
    first_cue: bool,*/
    runtime: SequenceRuntime,
}

impl SequenceRuntimePreset {
    pub fn new(id: u32, name: String, sequence: Sequence) -> Self {
        Self {
            id,
            name,
            runtime: SequenceRuntime::new(sequence),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_started(&self) -> bool {
        self.runtime.is_started()
    }

    pub fn channel_value(
        &self,
        fixture_id: u32,
        channel_id: u16,
    ) -> Option<FadeFixtureChannelValue> {
        self.runtime.channel_value(fixture_id, channel_id, 1.0, 1.0)
    }

    pub fn update(&mut self, delta_time: f64, fixture_handler: &mut FixtureHandler) {
        if self.runtime.update(delta_time, 1.0) {
            self.stop(fixture_handler);
        }
    }

    pub fn silent_start(&mut self) {
        self.runtime.start();
    }

    pub fn start(&mut self, fixture_handler: &mut FixtureHandler) {
        self.silent_start();

        for fixture_id in self
            .runtime
            .sequence()
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

    pub fn silent_stop(&mut self) {
        self.runtime.stop();
    }

    pub fn stop(&mut self, fixture_handler: &mut FixtureHandler) {
        self.silent_stop();

        for fixture_id in self
            .runtime
            .sequence()
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
}
