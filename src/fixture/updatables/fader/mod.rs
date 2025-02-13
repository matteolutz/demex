use config::{DemexFaderConfig, DemexFaderRuntimeFunction};
use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

pub mod config;
pub mod overrides;

use crate::fixture::{
    channel2::{channel_type::FixtureChannelType, channel_value::FixtureChannelValue2},
    error::FixtureError,
    handler::FixtureHandler,
    sequence::FadeFixtureChannelValue,
    value_source::{FixtureChannelValuePriority, FixtureChannelValueSource},
    Fixture,
};

use super::PresetHandler;

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

    pub fn set_value(&mut self, value: f32) {
        self.value = value;
    }

    pub fn home(&mut self, fixture_handler: &mut FixtureHandler) {
        self.value = 0.0;

        match &mut self.config {
            DemexFaderConfig::SequenceRuntime {
                fixtures,
                runtime,
                function: _,
            } => {
                runtime.stop();

                for fixture_id in fixtures {
                    fixture_handler
                        .fixture(*fixture_id)
                        .unwrap()
                        .remove_value_source(FixtureChannelValueSource::Fader {
                            fader_id: self.id,
                        });
                }
            }
            DemexFaderConfig::Submaster { fixtures } => {
                for fixture_id in fixtures {
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
            DemexFaderConfig::Submaster { fixtures: _ } => self.value != 0.0,
            DemexFaderConfig::SequenceRuntime {
                fixtures: _,
                function: _,
                runtime,
            } => runtime.is_started(),
        }
    }

    pub fn activate(&mut self, fixture_handler: &mut FixtureHandler) {
        match &mut self.config {
            DemexFaderConfig::Submaster { fixtures } => {
                for fixture_id in fixtures {
                    fixture_handler
                        .fixture(*fixture_id)
                        .unwrap()
                        .push_value_source(FixtureChannelValueSource::Fader { fader_id: self.id });
                }
            }
            DemexFaderConfig::SequenceRuntime {
                fixtures,
                runtime,
                function: _,
            } => {
                if !runtime.is_started() {
                    runtime.start();
                }

                for fixture_id in fixtures {
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
            DemexFaderConfig::Submaster { fixtures } => {
                if !fixtures.contains(&fixture.id())
                    || channel_type != FixtureChannelType::Intensity
                    || channel_type != FixtureChannelType::IntensityFine
                {
                    return Err(FixtureError::ChannelValueNotFound(channel_type));
                }

                Ok(FadeFixtureChannelValue::new(
                    FixtureChannelValue2::Discrete(255),
                    self.value,
                    self.priority,
                ))
            }
            DemexFaderConfig::SequenceRuntime {
                fixtures,
                runtime,
                function,
            } => {
                if !fixtures.contains(&fixture.id()) {
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

    pub fn update(&mut self, delta_time: f64, preset_handler: &PresetHandler) {
        match &mut self.config {
            DemexFaderConfig::SequenceRuntime {
                fixtures: _,
                runtime,
                function,
            } => {
                runtime.update(
                    delta_time,
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
