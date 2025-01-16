use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    fixture::{
        channel::{
            value::FixtureChannelDiscreteValue, FixtureChannel,
            FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
        },
        handler::FixtureHandler,
    },
    parser::nodes::fixture_selector::{FixtureSelector, FixtureSelectorContext},
};

use super::{error::PresetHandlerError, PresetHandler};

#[derive(Debug, Serialize, Deserialize)]
pub struct FixturePreset {
    id: u32,
    name: String,
    channel_type: u16,
    values: HashMap<u32, FixtureChannelDiscreteValue>,
}

impl FixturePreset {
    pub fn new(
        id: u32,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        channel_type: u16,
        preset_handler: &PresetHandler,
        fixture_handler: &FixtureHandler,
    ) -> Result<Self, PresetHandlerError> {
        let name = format!("{} Preset {}", FixtureChannel::name_by_id(channel_type), id);

        let mut values = HashMap::new();
        for fixture_id in fixture_selector
            .get_fixtures(preset_handler, fixture_selector_context)
            .map_err(|err| PresetHandlerError::FixtureSelectorError(Box::new(err)))?
        {
            let fixture = fixture_handler.fixture_immut(fixture_id);
            if let Some(fixture) = fixture {
                if !fixture
                    .channel_types()
                    .contains(&FIXTURE_CHANNEL_POSITION_PAN_TILT_ID)
                {
                    continue;
                }

                let fixture_position = fixture
                    .channel_value(channel_type, preset_handler)
                    .map_err(PresetHandlerError::FixtureError)?;

                values.insert(fixture_id, fixture_position.to_discrete());
            }
        }

        Ok(Self {
            id,
            name,
            channel_type,
            values,
        })
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

    pub fn channel_type(&self) -> u16 {
        self.channel_type
    }

    pub fn value(&self, fixture_id: u32) -> Option<&FixtureChannelDiscreteValue> {
        self.values.get(&fixture_id)
    }
}
