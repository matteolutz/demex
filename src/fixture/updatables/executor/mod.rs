use std::{collections::HashSet, time};

use config::ExecutorConfig;
use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::{
    effect::feature::runtime::FeatureEffectRuntime,
    gdtf::GdtfFixture,
    handler::{FixtureHandler, FixtureTypeList},
    presets::PresetHandler,
    selection::FixtureSelection,
    sequence::{runtime::SequenceRuntime, FadeFixtureChannelValue},
    timing::TimingHandler,
    value_source::{FixtureChannelValuePriority, FixtureChannelValueSource},
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

    #[serde(default)]
    fade_up: f32,

    config: ExecutorConfig,

    #[serde(default, skip_serializing, skip_deserializing)]
    #[egui_probe(skip)]
    started_at: Option<time::Instant>,
}

impl Executor {
    pub fn new_sequence(
        id: u32,
        name: Option<String>,
        sequence_id: u32,
        priority: FixtureChannelValuePriority,
    ) -> Self {
        Self {
            id,
            name: name.unwrap_or_else(|| format!("Sequence Executor {}", id)),
            config: ExecutorConfig::Sequence {
                runtime: SequenceRuntime::new(sequence_id),
            },
            priority,
            stomp_protected: false,
            fade_up: 0.0,
            started_at: None,
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
            fade_up: 0.0,
            started_at: None,
        }
    }

    pub fn config(&self) -> &ExecutorConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut ExecutorConfig {
        &mut self.config
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

    pub fn fixtures(&self, preset_handler: &PresetHandler) -> HashSet<u32> {
        match &self.config {
            ExecutorConfig::Sequence { runtime } => {
                let sequence = preset_handler.get_sequence(runtime.sequence_id()).unwrap();
                sequence.affected_fixtures(preset_handler)
            }
            ExecutorConfig::FeatureEffect { selection, .. } => {
                selection.fixtures().iter().copied().collect()
            }
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
        fixture_types: &FixtureTypeList,
        fixture: &GdtfFixture,
        channel: &gdtf::dmx_mode::DmxChannel,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> Option<FadeFixtureChannelValue> {
        let started_delta = self
            .started_at
            .map(|started_at| started_at.elapsed().as_secs_f32())
            .unwrap_or(0.0);
        let fade = if self.fade_up > 0.0 {
            (started_delta / self.fade_up).clamp(0.0, 1.0)
        } else {
            1.0
        };

        match &self.config {
            ExecutorConfig::Sequence { runtime } => {
                let fixtures = self.fixtures(preset_handler);

                if !fixtures.contains(&fixture.id()) {
                    None
                } else {
                    runtime
                        .channel_value(
                            fixture_types,
                            fixture,
                            channel,
                            1.0,
                            1.0,
                            preset_handler,
                            timing_handler,
                            self.priority,
                        )
                        .map(|val| val.multiply(fade))
                }
            }
            ExecutorConfig::FeatureEffect { runtime, selection } => {
                if !selection.has_fixture(fixture.id()) {
                    None
                } else {
                    runtime
                        .get_channel_value(
                            channel.name().as_ref(),
                            fixture,
                            fixture_types,
                            selection.offset(fixture.id())?,
                            self.priority,
                            timing_handler,
                        )
                        .ok()
                        .map(|val| val.multiply(fade))
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
                    self.stop(fixture_handler, preset_handler);
                }
            }
            ExecutorConfig::FeatureEffect { .. } => {}
        }
    }

    fn child_start(&mut self, time_offset: f32) {
        match &mut self.config {
            ExecutorConfig::Sequence { runtime, .. } => runtime.start(time_offset),
            ExecutorConfig::FeatureEffect { runtime, .. } => runtime.start(time_offset),
        }
    }

    pub fn start(
        &mut self,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
        time_offset: f32,
    ) {
        self.child_start(time_offset);

        self.started_at = Some(time::Instant::now() - time::Duration::from_secs_f32(time_offset));

        for fixture_id in self.fixtures(preset_handler) {
            if let Some(fixture) = fixture_handler.fixture(fixture_id) {
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

    pub fn stop(&mut self, fixture_handler: &mut FixtureHandler, preset_handler: &PresetHandler) {
        self.child_stop();

        for fixture_id in self.fixtures(preset_handler) {
            if let Some(fixture) = fixture_handler.fixture(fixture_id) {
                fixture.remove_value_source(FixtureChannelValueSource::Executor {
                    executor_id: self.id,
                });
            }
        }
    }

    pub fn next_cue(
        &mut self,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
        time_offset: f32,
    ) {
        if let ExecutorConfig::Sequence { runtime, .. } = &mut self.config {
            if runtime.next_cue(preset_handler, time_offset) {
                self.stop(fixture_handler, preset_handler);
            }
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
