use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::{
    channel::value_source::{FixtureChannelValuePriority, FixtureChannelValueSource},
    handler::FixtureHandler,
    presets::PresetHandler,
    sequence::{runtime::SequenceRuntime, FadeFixtureChannelValue},
};

#[derive(Debug, Clone, Serialize, Deserialize, EguiProbe)]
pub struct SequenceRuntimeExecutor {
    #[egui_probe(skip)]
    id: u32,

    #[serde(default)]
    priority: FixtureChannelValuePriority,

    #[serde(default)]
    stop_others: bool,

    runtime: SequenceRuntime,

    fixtures: Vec<u32>,
}

impl SequenceRuntimeExecutor {
    pub fn new(
        id: u32,
        sequence_id: u32,
        fixtures: Vec<u32>,
        priority: FixtureChannelValuePriority,
    ) -> Self {
        Self {
            id,
            runtime: SequenceRuntime::new(sequence_id),
            fixtures,
            priority,
            stop_others: false,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self, preset_handler: &PresetHandler) -> String {
        preset_handler
            .get_sequence(self.runtime.sequence_id())
            .unwrap()
            .name()
            .to_owned()
    }

    pub fn runtime(&self) -> &SequenceRuntime {
        &self.runtime
    }

    pub fn stop_others(&self) -> bool {
        self.stop_others
    }

    pub fn is_started(&self) -> bool {
        self.runtime.is_started()
    }

    pub fn fixtures(&self) -> &Vec<u32> {
        &self.fixtures
    }

    pub fn channel_value(
        &self,
        fixture_id: u32,
        channel_id: u16,
        preset_handler: &PresetHandler,
    ) -> Option<FadeFixtureChannelValue> {
        self.runtime.channel_value(
            fixture_id,
            channel_id,
            1.0,
            1.0,
            preset_handler,
            self.priority,
        )
    }

    pub fn update(
        &mut self,
        delta_time: f64,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
    ) {
        if self.runtime.update(delta_time, 1.0, preset_handler) {
            self.stop(fixture_handler);
        }
    }

    fn silent_start(&mut self) {
        self.runtime.start();
    }

    pub fn start(&mut self, fixture_handler: &mut FixtureHandler) {
        self.silent_start();

        for fixture_id in &self.fixtures {
            if let Some(fixture) = fixture_handler.fixture(*fixture_id) {
                fixture.push_value_source(FixtureChannelValueSource::Executor {
                    executor_id: self.id,
                });
            }
        }
    }

    fn silent_stop(&mut self) {
        self.runtime.stop();
    }

    pub fn stop(&mut self, fixture_handler: &mut FixtureHandler) {
        self.silent_stop();

        for fixture_id in &self.fixtures {
            if let Some(fixture) = fixture_handler.fixture(*fixture_id) {
                fixture.remove_value_source(FixtureChannelValueSource::Executor {
                    executor_id: self.id,
                });
            }
        }
    }

    pub fn next_cue(&mut self, preset_handler: &PresetHandler) {
        self.runtime.next_cue(preset_handler);
    }
}
