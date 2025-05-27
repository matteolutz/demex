use std::collections::HashSet;

use config::DemexFaderRuntimeFunction;
use serde::{Deserialize, Serialize};

pub mod config;

use crate::fixture::{
    error::FixtureError,
    gdtf::GdtfFixture,
    handler::{FixtureHandler, FixtureTypeList},
    presets::PresetHandler,
    sequence::{runtime::SequenceRuntime, FadeFixtureChannelValue},
    timing::TimingHandler,
    value_source::{FixtureChannelValuePriority, FixtureChannelValueSource},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub struct DemexFader {
    #[cfg_attr(feature = "ui", egui_probe(skip))]
    id: u32,

    priority: FixtureChannelValuePriority,

    #[serde(default)]
    stomp_protected: bool,

    #[serde(default, skip_serializing)]
    #[cfg_attr(feature = "ui", egui_probe(skip))]
    value: f32,

    runtime: SequenceRuntime,
    function: DemexFaderRuntimeFunction,
}

impl DemexFader {
    pub fn new(id: u32, runtime: SequenceRuntime, function: DemexFaderRuntimeFunction) -> Self {
        Self {
            id,
            runtime,
            function,
            priority: FixtureChannelValuePriority::Ltp,
            value: 0.0,
            stomp_protected: false,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn stomp_protected(&self) -> bool {
        self.stomp_protected
    }

    pub fn display_name(&self, preset_handler: &PresetHandler) -> String {
        let sequence_name = preset_handler
            .get_sequence(self.runtime.sequence_id())
            .map(|seq| seq.name());
        format!("{}", sequence_name.unwrap_or("[Deleted Sequence]"))
    }

    pub fn priority(&self) -> FixtureChannelValuePriority {
        self.priority
    }

    pub fn runtime(&self) -> &SequenceRuntime {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut SequenceRuntime {
        &mut self.runtime
    }

    pub fn function(&self) -> DemexFaderRuntimeFunction {
        self.function
    }

    pub fn function_mut(&mut self) -> &mut DemexFaderRuntimeFunction {
        &mut self.function
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn go(
        &mut self,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
        time_offset: f32,
    ) {
        if !self.is_active() {
            self.start(fixture_handler, preset_handler, time_offset);
            return;
        }

        if self.runtime.next_cue(preset_handler, time_offset) {
            self.stop(fixture_handler, preset_handler);
        }
    }

    pub fn set_value(
        &mut self,
        value: f32,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
        time_offset: f32,
    ) {
        if value == 0.0 {
            self.stop(fixture_handler, preset_handler);
            return;
        }

        if !self.is_active() {
            self.start(fixture_handler, preset_handler, time_offset);
        }

        self.value = value;
    }

    pub fn is_active(&self) -> bool {
        self.runtime.is_started()
    }

    pub fn fixtures(&self, preset_handler: &PresetHandler) -> HashSet<u32> {
        let sequence = preset_handler
            .get_sequence(self.runtime.sequence_id())
            .unwrap();
        sequence.affected_fixtures(preset_handler)
    }

    pub fn start(
        &mut self,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
        time_offset: f32,
    ) {
        self.value = 1.0;
        self.runtime.start(time_offset);

        // self.started_at = Some(time::Instant::now() - time::Duration::from_secs_f32(time_offset));

        for fixture_id in self.fixtures(preset_handler) {
            if let Some(fixture) = fixture_handler.fixture(fixture_id) {
                fixture.push_value_source(FixtureChannelValueSource::Executor {
                    executor_id: self.id,
                });
            }
        }
    }

    pub fn stop(&mut self, fixture_handler: &mut FixtureHandler, preset_handler: &PresetHandler) {
        self.value = 0.0;
        self.runtime.stop();

        for fixture_id in self.fixtures(preset_handler) {
            if let Some(fixture) = fixture_handler.fixture(fixture_id) {
                fixture.remove_value_source(FixtureChannelValueSource::Executor {
                    executor_id: self.id,
                });
            }
        }
    }

    pub fn channel_value(
        &self,
        fixture_types: &FixtureTypeList,
        fixture: &GdtfFixture,
        channel: &gdtf::dmx_mode::DmxChannel,
        preset_handler: &PresetHandler,
        _timing_handler: &TimingHandler,
    ) -> Result<FadeFixtureChannelValue, FixtureError> {
        if !self.is_active() {
            return Err(FixtureError::GdtfChannelValueNotFound(
                channel.name().as_ref().to_owned(),
            ));
        }

        let sequence = preset_handler
            .get_sequence(self.runtime.sequence_id())
            .unwrap();
        let fixtures = sequence.affected_fixtures(preset_handler);

        if !fixtures.contains(&fixture.id()) {
            return Err(FixtureError::GdtfChannelValueNotFound(
                channel.name().as_ref().to_owned(),
            ));
        }

        let _speed_multiplier = if self.function == DemexFaderRuntimeFunction::Speed {
            self.value
        } else {
            1.0
        };

        let intensity_multiplier = if self.function == DemexFaderRuntimeFunction::Intensity {
            self.value
        } else {
            1.0
        };

        let channel_attribute = channel.logical_channels[0]
            .attribute(fixture.fixture_type_and_dmx_mode(fixture_types).unwrap().0);

        self.runtime
            .channel_value(fixture, channel, self.priority)
            .map(|value| {
                if self.function == DemexFaderRuntimeFunction::FadeAll {
                    return value.multiply(self.value);
                }

                if channel_attribute
                    .and_then(|attribute| attribute.name.as_ref())
                    .is_some_and(|attribute_name| attribute_name.as_ref() == "Dimmer")
                {
                    value.multiply(intensity_multiplier)
                } else {
                    value
                }
            })
            .ok_or(FixtureError::GdtfChannelValueNotFound(
                channel.name().as_ref().to_owned(),
            ))
    }

    pub fn update(
        &mut self,
        fixture_types: &FixtureTypeList,
        fixture_handler: &FixtureHandler,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) {
        self.runtime.update(
            if self.function == DemexFaderRuntimeFunction::Speed {
                self.value
            } else {
                1.0
            },
            fixture_types,
            fixture_handler,
            preset_handler,
            timing_handler,
            self.priority,
        );
    }
}
