use config::{DemexFaderConfig, DemexFaderRuntimeFunction};
use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

pub mod config;

use crate::fixture::{
    error::FixtureError,
    gdtf::GdtfFixture,
    handler::{FixtureHandler, FixtureTypeList},
    presets::PresetHandler,
    sequence::FadeFixtureChannelValue,
    timing::TimingHandler,
    value_source::{FixtureChannelValuePriority, FixtureChannelValueSource},
};

use super::error::UpdatableHandlerError;

#[derive(Debug, Serialize, Deserialize, Clone, EguiProbe)]
pub struct DemexFader {
    #[egui_probe(skip)]
    id: u32,

    name: String,

    priority: FixtureChannelValuePriority,

    #[serde(default)]
    stomp_protected: bool,

    #[serde(default, skip_serializing)]
    #[egui_probe(skip)]
    value: f32,

    config: DemexFaderConfig,
}

impl DemexFader {
    pub fn new(id: u32, name: String, config: DemexFaderConfig) -> Self {
        Self {
            id,
            name,
            config,
            priority: FixtureChannelValuePriority::Ltp,
            value: 0.0,
            stomp_protected: false,
        }
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

    pub fn display_name(&self, preset_handler: &PresetHandler) -> String {
        match &self.config {
            DemexFaderConfig::SequenceRuntime { runtime, .. } => {
                let sequence_name = preset_handler
                    .get_sequence(runtime.sequence_id())
                    .map(|seq| seq.name());
                format!("{} ({})", self.name(), sequence_name.unwrap_or("Deleted"))
            }
        }
    }

    pub fn priority(&self) -> FixtureChannelValuePriority {
        self.priority
    }

    pub fn config(&self) -> &DemexFaderConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut DemexFaderConfig {
        &mut self.config
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn sequence_go(
        &mut self,
        preset_handler: &PresetHandler,
    ) -> Result<(), UpdatableHandlerError> {
        let is_active = self.is_active();
        match &mut self.config {
            DemexFaderConfig::SequenceRuntime { runtime, .. } => {
                if is_active {
                    runtime.next_cue(preset_handler, 0.0);
                }

                Ok(())
            }
        }
    }

    pub fn set_value(
        &mut self,
        value: f32,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
    ) {
        if value == 0.0 {
            self.home(fixture_handler, preset_handler);
            return;
        }

        if !self.is_active() {
            self.activate(fixture_handler, preset_handler);
        }

        self.value = value;
    }

    pub fn home(&mut self, fixture_handler: &mut FixtureHandler, preset_handler: &PresetHandler) {
        self.value = 0.0;

        match &mut self.config {
            DemexFaderConfig::SequenceRuntime { runtime, .. } => {
                runtime.stop();

                let sequence = preset_handler.get_sequence(runtime.sequence_id()).unwrap();

                for fixture_id in sequence.affected_fixtures(preset_handler) {
                    fixture_handler
                        .fixture(fixture_id)
                        .unwrap()
                        .remove_value_source(FixtureChannelValueSource::Fader {
                            fader_id: self.id,
                        });
                }
            }
        }
    }

    pub fn is_active(&self) -> bool {
        match &self.config {
            DemexFaderConfig::SequenceRuntime { runtime, .. } => runtime.is_started(),
        }
    }

    fn activate(&mut self, fixture_handler: &mut FixtureHandler, preset_handler: &PresetHandler) {
        match &mut self.config {
            DemexFaderConfig::SequenceRuntime { runtime, .. } => {
                let sequence = preset_handler.get_sequence(runtime.sequence_id()).unwrap();

                if !runtime.is_started() {
                    runtime.start(0.0);
                }

                for fixture_id in sequence.affected_fixtures(preset_handler) {
                    fixture_handler
                        .fixture(fixture_id)
                        .unwrap()
                        .push_value_source(FixtureChannelValueSource::Fader { fader_id: self.id });
                }
            }
        }
    }

    pub fn get_channel_value(
        &self,
        fixture_types: &FixtureTypeList,
        fixture: &GdtfFixture,
        channel: &gdtf::dmx_mode::DmxChannel,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> Result<FadeFixtureChannelValue, FixtureError> {
        if !self.is_active() {
            return Err(FixtureError::GdtfChannelValueNotFound(
                channel.name().as_ref().to_owned(),
            ));
        }

        match &self.config {
            DemexFaderConfig::SequenceRuntime { runtime, function } => {
                let sequence = preset_handler.get_sequence(runtime.sequence_id()).unwrap();
                let fixtures = sequence.affected_fixtures(preset_handler);

                if !fixtures.contains(&fixture.id()) {
                    return Err(FixtureError::GdtfChannelValueNotFound(
                        channel.name().as_ref().to_owned(),
                    ));
                }

                let speed_multiplier = if *function == DemexFaderRuntimeFunction::Speed {
                    self.value
                } else {
                    1.0
                };

                let intensity_multiplier = if *function == DemexFaderRuntimeFunction::Intensity {
                    self.value
                } else {
                    1.0
                };

                runtime
                    .channel_value(
                        fixture_types,
                        fixture,
                        channel,
                        speed_multiplier,
                        intensity_multiplier,
                        preset_handler,
                        timing_handler,
                        self.priority,
                    )
                    .ok_or(FixtureError::GdtfChannelValueNotFound(
                        channel.name().as_ref().to_owned(),
                    ))
            }
        }
    }

    pub fn update(&mut self, _delta_time: f64, preset_handler: &PresetHandler) {
        match &mut self.config {
            DemexFaderConfig::SequenceRuntime {
                runtime, function, ..
            } => {
                runtime.update(
                    if *function == DemexFaderRuntimeFunction::Speed {
                        self.value
                    } else {
                        1.0
                    },
                    preset_handler,
                );
            }
        }
    }
}
