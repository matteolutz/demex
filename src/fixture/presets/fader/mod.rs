use config::DemexFaderConfig;
use serde::{Deserialize, Serialize};

pub mod config;

use crate::fixture::{
    channel::{
        value::{FixtureChannelDiscreteValue, FixtureChannelValue},
        FIXTURE_CHANNEL_INTENSITY_ID,
    },
    error::FixtureError,
    handler::FixtureHandler,
    presets::PresetHandler,
    sequence::FadeFixtureChannelValue,
    value_source::{FixtureChannelValuePriority, FixtureChannelValueSource},
    Fixture,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DemexFader {
    id: u32,
    name: String,

    priority: FixtureChannelValuePriority,

    #[serde(default, skip_serializing)]
    value: f32,

    config: DemexFaderConfig,
}

impl DemexFader {
    pub fn new(id: u32, name: String, config: DemexFaderConfig) -> Self {
        Self {
            id,
            name,
            config,
            priority: FixtureChannelValuePriority::LTP,
            value: 0.0,
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

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value;
    }

    pub fn home(&mut self) {
        self.value = 0.0;
    }

    pub fn is_active(&self, preset_handler: &PresetHandler) -> bool {
        match self.config {
            DemexFaderConfig::Submaster { fixtures: _ } => self.value != 0.0,
            DemexFaderConfig::SequenceRuntime {
                fixtures: _,
                runtime_id,
            } => preset_handler
                .sequence_runtime(runtime_id)
                .unwrap()
                .is_started(),
        }
    }

    pub fn activate(
        &self,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &mut PresetHandler,
    ) {
        match &self.config {
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
                runtime_id,
            } => {
                if !preset_handler
                    .sequence_runtime(*runtime_id)
                    .unwrap()
                    .is_started()
                {
                    preset_handler
                        .sequence_runtime_mut(*runtime_id)
                        .unwrap()
                        .silent_start();
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
        preset_handler: &PresetHandler,
    ) -> Result<FadeFixtureChannelValue, FixtureError> {
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
                runtime_id,
            } => {
                if !fixtures.contains(&fixture.id()) {
                    return Err(FixtureError::ChannelValueNotFound(channel_id));
                }

                let runtime = preset_handler.sequence_runtime(*runtime_id).unwrap();

                runtime
                    .channel_value(fixture.id(), channel_id, self.value)
                    .ok_or(FixtureError::ChannelValueNotFound(channel_id))
            }
        }
    }
}
