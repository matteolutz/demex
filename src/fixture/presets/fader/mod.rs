use std::collections::HashMap;

use config::{DemexFaderConfig, DemexFaderRuntimeFunction};
use egui_probe::EguiProbe;
use overrides::DemexFaderChannelOverride;
use serde::{Deserialize, Serialize};

pub mod config;
pub mod overrides;

use crate::fixture::{
    channel::{
        value::{FixtureChannelDiscreteValue, FixtureChannelValue},
        FIXTURE_CHANNEL_INTENSITY_ID,
    },
    error::FixtureError,
    handler::FixtureHandler,
    sequence::FadeFixtureChannelValue,
    value_source::{FixtureChannelValuePriority, FixtureChannelValueSource},
    Fixture,
};

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

    #[serde(default)]
    overrides: HashMap<u16, DemexFaderChannelOverride>,
}

impl DemexFader {
    pub fn new(id: u32, name: String, config: DemexFaderConfig) -> Self {
        Self {
            id,
            name,
            config,
            priority: FixtureChannelValuePriority::LTP,
            value: 0.0,
            overrides: HashMap::new(),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn priority(&self) -> FixtureChannelValuePriority {
        self.priority
    }

    pub fn config(&self) -> &DemexFaderConfig {
        &self.config
    }

    pub fn overrides(&self) -> &HashMap<u16, DemexFaderChannelOverride> {
        &self.overrides
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
        channel_id: u16,
    ) -> Result<FadeFixtureChannelValue, FixtureError> {
        if !self.is_active() {
            return Err(FixtureError::ChannelValueNotFound(channel_id));
        }

        if let Some(channel_override) = self.overrides.get(&channel_id) {
            return if let Some(from_value) = channel_override.from_value() {
                Ok(FadeFixtureChannelValue::new(
                    FixtureChannelValue::Mix {
                        a: Box::new(from_value.clone()),
                        b: Box::new(channel_override.to_value().clone()),
                        mix: self.value,
                    },
                    1.0,
                ))
            } else {
                Ok(FadeFixtureChannelValue::new(
                    channel_override.to_value().clone(),
                    self.value,
                ))
            };
        }

        match &self.config {
            DemexFaderConfig::Submaster { fixtures } => {
                if !fixtures.contains(&fixture.id()) || channel_id != FIXTURE_CHANNEL_INTENSITY_ID {
                    return Err(FixtureError::ChannelValueNotFound(channel_id));
                }

                Ok(FadeFixtureChannelValue::new(
                    FixtureChannelValue::Discrete(FixtureChannelDiscreteValue::Single(1.0)),
                    self.value,
                ))
            }
            DemexFaderConfig::SequenceRuntime {
                fixtures,
                runtime,
                function,
            } => {
                if !fixtures.contains(&fixture.id()) {
                    return Err(FixtureError::ChannelValueNotFound(channel_id));
                }

                let speed_multiplier = if function == &DemexFaderRuntimeFunction::Speed {
                    self.value
                } else {
                    1.0
                };
                let intensity_multiplier = if function == &DemexFaderRuntimeFunction::Intensity {
                    self.value
                } else {
                    1.0
                };

                runtime
                    .channel_value(
                        fixture.id(),
                        channel_id,
                        speed_multiplier,
                        intensity_multiplier,
                    )
                    .ok_or(FixtureError::ChannelValueNotFound(channel_id))
            }
        }
    }

    pub fn update(&mut self, delta_time: f64) {
        match &mut self.config {
            DemexFaderConfig::SequenceRuntime {
                fixtures: _,
                runtime,
                function: _,
            } => {
                runtime.update(delta_time, self.value);
            }
            _ => {}
        }
    }
}
