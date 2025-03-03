use config::{DemexFaderConfig, DemexFaderRuntimeFunction};
use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

pub mod config;

use crate::{
    fixture::{
        channel2::{channel_type::FixtureChannelType, channel_value::FixtureChannelValue2},
        error::FixtureError,
        handler::FixtureHandler,
        sequence::FadeFixtureChannelValue,
        value_source::{FixtureChannelValuePriority, FixtureChannelValueSource},
        Fixture,
    },
    utils::math::f32_to_coarse_fine,
};

use super::{error::UpdatableHandlerError, PresetHandler};

#[derive(Debug, Serialize, Deserialize, Clone, EguiProbe)]
pub struct DemexFader {
    #[egui_probe(skip)]
    id: u32,

    name: String,

    priority: FixtureChannelValuePriority,

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

    pub fn display_name(&self, preset_handler: &PresetHandler) -> String {
        match &self.config {
            DemexFaderConfig::Submaster { .. } => self.name().to_owned(),
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
                    runtime.next_cue(preset_handler);
                }

                Ok(())
            }
            _ => Err(UpdatableHandlerError::FaderIsNotASequence(self.id)),
        }
    }

    pub fn set_value(&mut self, value: f32, fixture_handler: &mut FixtureHandler) {
        if value == 0.0 {
            self.home(fixture_handler);
            return;
        }

        if !self.is_active() {
            self.activate(fixture_handler);
        }

        self.value = value;
    }

    pub fn home(&mut self, fixture_handler: &mut FixtureHandler) {
        self.value = 0.0;

        match &mut self.config {
            DemexFaderConfig::SequenceRuntime {
                selection: fixtures,
                runtime,
                ..
            } => {
                runtime.stop();

                for fixture_id in fixtures.fixtures() {
                    fixture_handler
                        .fixture(*fixture_id)
                        .unwrap()
                        .remove_value_source(FixtureChannelValueSource::Fader {
                            fader_id: self.id,
                        });
                }
            }
            DemexFaderConfig::Submaster {
                selection: fixtures,
            } => {
                for fixture_id in fixtures.fixtures() {
                    fixture_handler
                        .fixture(*fixture_id)
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
            DemexFaderConfig::Submaster { selection: _ } => self.value != 0.0,
            DemexFaderConfig::SequenceRuntime { runtime, .. } => runtime.is_started(),
        }
    }

    fn activate(&mut self, fixture_handler: &mut FixtureHandler) {
        match &mut self.config {
            DemexFaderConfig::Submaster {
                selection: fixtures,
            } => {
                for fixture_id in fixtures.fixtures() {
                    fixture_handler
                        .fixture(*fixture_id)
                        .unwrap()
                        .push_value_source(FixtureChannelValueSource::Fader { fader_id: self.id });
                }
            }
            DemexFaderConfig::SequenceRuntime {
                selection: fixtures,
                runtime,
                ..
            } => {
                if !runtime.is_started() {
                    runtime.start();
                }

                for fixture_id in fixtures.fixtures() {
                    fixture_handler
                        .fixture(*fixture_id)
                        .unwrap()
                        .push_value_source(FixtureChannelValueSource::Fader { fader_id: self.id });
                }
            }
        }
    }

    pub fn get_channel_value(
        &self,
        fixture: &Fixture,
        channel_type: FixtureChannelType,
        preset_handler: &PresetHandler,
    ) -> Result<FadeFixtureChannelValue, FixtureError> {
        if !self.is_active() {
            return Err(FixtureError::ChannelValueNotFound(channel_type));
        }

        match &self.config {
            DemexFaderConfig::Submaster {
                selection: fixtures,
            } => {
                if !fixtures.has_fixture(fixture.id())
                    || (channel_type != FixtureChannelType::Intensity
                        && channel_type != FixtureChannelType::IntensityFine)
                {
                    return Err(FixtureError::ChannelValueNotFound(channel_type));
                }

                let (coarse, fine) = f32_to_coarse_fine(self.value);

                let value = match channel_type {
                    FixtureChannelType::Intensity => coarse,
                    FixtureChannelType::IntensityFine => fine,
                    _ => unreachable!(),
                };

                Ok(FadeFixtureChannelValue::new(
                    FixtureChannelValue2::Discrete(value),
                    1.0,
                    self.priority,
                ))
            }
            DemexFaderConfig::SequenceRuntime {
                selection: fixtures,
                runtime,
                function,
            } => {
                if !fixtures.has_fixture(fixture.id()) {
                    return Err(FixtureError::ChannelValueNotFound(channel_type));
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
                        fixture.id(),
                        channel_type,
                        speed_multiplier,
                        intensity_multiplier,
                        preset_handler,
                        self.priority,
                    )
                    .ok_or(FixtureError::ChannelValueNotFound(channel_type))
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
            _ => {}
        }
    }
}
