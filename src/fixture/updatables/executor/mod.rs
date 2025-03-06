use config::ExecutorConfig;
use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::{
    channel2::{channel_type::FixtureChannelType, feature::feature_config::FixtureFeatureConfig},
    effect::feature::runtime::FeatureEffectRuntime,
    handler::FixtureHandler,
    presets::PresetHandler,
    selection::FixtureSelection,
    sequence::{runtime::SequenceRuntime, FadeFixtureChannelValue},
    timing::TimingHandler,
    value_source::{FixtureChannelValuePriority, FixtureChannelValueSource},
    Fixture,
};

pub mod config;

#[derive(Debug, Clone, Serialize, Deserialize, EguiProbe)]
pub struct Executor {
    #[egui_probe(skip)]
    id: u32,

    #[serde(default)]
    name: String,

    #[serde(default)]
    priority: FixtureChannelValuePriority,

    #[serde(default)]
    stomp_protected: bool,

    config: ExecutorConfig,
}

impl Executor {
    pub fn new_sequence(
        id: u32,
        name: Option<String>,
        sequence_id: u32,
        selection: FixtureSelection,
        priority: FixtureChannelValuePriority,
    ) -> Self {
        Self {
            id,
            name: name.unwrap_or_else(|| format!("Sequence Executor {}", id)),
            config: ExecutorConfig::Sequence {
                runtime: SequenceRuntime::new(sequence_id),
                fixtures: selection.fixtures().to_vec(),
            },
            priority,
            stomp_protected: false,
        }
    }

    pub fn new_effect(
        id: u32,
        name: Option<String>,
        selection: FixtureSelection,
        priority: FixtureChannelValuePriority,
    ) -> Self {
        Self {
            id,
            name: name.unwrap_or_else(|| format!("Effect Executor {}", id)),
            config: ExecutorConfig::FeatureEffect {
                runtime: FeatureEffectRuntime::default(),
                selection,
            },
            priority,
            stomp_protected: false,
        }
    }

    pub fn config(&self) -> &ExecutorConfig {
        &self.config
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    pub fn stomp_protected(&self) -> bool {
        self.stomp_protected
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
        self.stomp_protected
    }

    pub fn is_started(&self) -> bool {
        match &self.config {
            ExecutorConfig::Sequence { runtime, .. } => runtime.is_started(),
            ExecutorConfig::FeatureEffect { runtime, .. } => runtime.is_started(),
        }
    }

    pub fn channel_value(
        &self,
        fixture: &Fixture,
        fixture_feature_configs: &[FixtureFeatureConfig],
        channel_type: FixtureChannelType,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> Option<FadeFixtureChannelValue> {
        match &self.config {
            ExecutorConfig::Sequence { runtime, fixtures } => {
                if !fixtures.contains(&fixture.id()) {
                    None
                } else {
                    runtime.channel_value(
                        fixture,
                        channel_type,
                        1.0,
                        1.0,
                        preset_handler,
                        timing_handler,
                        self.priority,
                    )
                }
            }
            ExecutorConfig::FeatureEffect { runtime, selection } => {
                if !selection.has_fixture(fixture.id()) {
                    None
                } else {
                    runtime.get_channel_value(
                        channel_type,
                        fixture_feature_configs,
                        selection.offset(fixture.id())?,
                        self.priority,
                        timing_handler,
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
                "{}\nSeq - ({}/{})\n{}",
                self.name(),
                runtime
                    .current_cue()
                    .map(|c| (c + 1).to_string())
                    .unwrap_or("-".to_owned()),
                runtime.num_cues(preset_handler),
                preset_handler
                    .get_sequence(runtime.sequence_id())
                    .unwrap()
                    .name()
            ),
            ExecutorConfig::FeatureEffect { runtime, .. } => {
                format!("{}\nEffect\n{}", self.name(), runtime.effect())
            }
        }
    }
}
