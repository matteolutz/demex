use config::ExecutorConfig;
use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::{
    channel2::{channel_type::FixtureChannelType, feature::feature_config::FixtureFeatureConfig},
    handler::FixtureHandler,
    presets::PresetHandler,
    sequence::{runtime::SequenceRuntime, FadeFixtureChannelValue},
    value_source::{FixtureChannelValuePriority, FixtureChannelValueSource},
};

pub mod config;

#[derive(Debug, Clone, Serialize, Deserialize, EguiProbe)]
pub struct Executor {
    #[egui_probe(skip)]
    id: u32,

    #[serde(default)]
    priority: FixtureChannelValuePriority,

    #[serde(default)]
    stop_others: bool,

    config: ExecutorConfig,
}

impl Executor {
    pub fn new(
        id: u32,
        sequence_id: u32,
        fixtures: Vec<u32>,
        priority: FixtureChannelValuePriority,
    ) -> Self {
        Self {
            id,
            config: ExecutorConfig::Sequence {
                runtime: SequenceRuntime::new(sequence_id),
                fixtures,
            },
            priority,
            stop_others: false,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self, preset_handler: &PresetHandler) -> String {
        match &self.config {
            ExecutorConfig::Sequence { runtime, .. } => preset_handler
                .get_sequence(runtime.sequence_id())
                .unwrap()
                .name()
                .to_owned(),
            ExecutorConfig::FeatureEffect { runtime, .. } => format!("{}", runtime.effect()),
        }
    }

    pub fn fixtures(&self) -> &[u32] {
        match &self.config {
            ExecutorConfig::Sequence { fixtures, .. } => fixtures,
            ExecutorConfig::FeatureEffect { selection, .. } => selection.fixtures(),
        }
    }

    pub fn refers_to_sequence(&self, sequence_id: u32) -> bool {
        if let ExecutorConfig::Sequence { runtime, .. } = &self.config {
            runtime.sequence_id() == sequence_id
        } else {
            false
        }
    }

    pub fn stop_others(&self) -> bool {
        self.stop_others
    }

    pub fn is_started(&self) -> bool {
        match &self.config {
            ExecutorConfig::Sequence { runtime, .. } => runtime.is_started(),
            ExecutorConfig::FeatureEffect { runtime, .. } => runtime.is_started(),
        }
    }

    pub fn channel_value(
        &self,
        fixture_id: u32,
        fixture_feature_configs: &[FixtureFeatureConfig],
        channel_type: FixtureChannelType,
        preset_handler: &PresetHandler,
    ) -> Option<FadeFixtureChannelValue> {
        match &self.config {
            ExecutorConfig::Sequence { runtime, fixtures } => {
                if !fixtures.contains(&fixture_id) {
                    None
                } else {
                    runtime.channel_value(
                        fixture_id,
                        channel_type,
                        1.0,
                        1.0,
                        preset_handler,
                        self.priority,
                    )
                }
            }
            ExecutorConfig::FeatureEffect { runtime, selection } => {
                if !selection.has_fixture(fixture_id) {
                    None
                } else {
                    runtime.get_channel_value(
                        channel_type,
                        fixture_feature_configs,
                        selection.offset_idx(fixture_id)?,
                        self.priority,
                    )
                }
            }
        }
    }

    pub fn update(
        &mut self,
        _delta_time: f64,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
    ) {
        match &mut self.config {
            ExecutorConfig::Sequence { runtime, .. } => {
                if runtime.update(1.0, preset_handler) {
                    self.stop(fixture_handler);
                }
            }
            ExecutorConfig::FeatureEffect { .. } => {}
        }
    }

    fn child_start(&mut self) {
        match &mut self.config {
            ExecutorConfig::Sequence { runtime, .. } => runtime.start(),
            ExecutorConfig::FeatureEffect { runtime, .. } => runtime.start(),
        }
    }

    pub fn start(&mut self, fixture_handler: &mut FixtureHandler) {
        self.child_start();

        for fixture_id in self.fixtures() {
            if let Some(fixture) = fixture_handler.fixture(*fixture_id) {
                fixture.push_value_source(FixtureChannelValueSource::Executor {
                    executor_id: self.id,
                });
            }
        }
    }

    fn child_stop(&mut self) {
        match &mut self.config {
            ExecutorConfig::Sequence { runtime, .. } => runtime.stop(),
            ExecutorConfig::FeatureEffect { runtime, .. } => runtime.stop(),
        }
    }

    pub fn stop(&mut self, fixture_handler: &mut FixtureHandler) {
        self.child_stop();

        for fixture_id in self.fixtures() {
            if let Some(fixture) = fixture_handler.fixture(*fixture_id) {
                fixture.remove_value_source(FixtureChannelValueSource::Executor {
                    executor_id: self.id,
                });
            }
        }
    }

    pub fn next_cue(&mut self, preset_handler: &PresetHandler) {
        if let ExecutorConfig::Sequence { runtime, .. } = &mut self.config {
            runtime.next_cue(preset_handler);
        }
    }

    pub fn to_string(&self, preset_handler: &PresetHandler) -> String {
        match &self.config {
            ExecutorConfig::Sequence { runtime, .. } => format!(
                "{}\nSeq - ({}/{})",
                self.name(preset_handler),
                runtime
                    .current_cue()
                    .map(|c| (c + 1).to_string())
                    .unwrap_or("-".to_owned()),
                runtime.num_cues(preset_handler),
            ),
            ExecutorConfig::FeatureEffect { .. } => {
                format!("{}\nEffect", self.name(preset_handler))
            }
        }
    }
}
