use std::collections::HashMap;

use crate::{
    fixture::{channel::FIXTURE_CHANNEL_POSITION_PAN_TILT_ID, handler::FixtureHandler},
    parser::nodes::fixture_selector::{FixtureSelector, FixtureSelectorContext},
};

use super::{error::PresetHandlerError, PresetHandler};

#[derive(Debug)]
pub struct FixturePositionPreset {
    id: u32,
    name: String,
    position: HashMap<u32, [f32; 2]>,
}

impl FixturePositionPreset {
    pub fn new(
        id: u32,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        preset_handler: &PresetHandler,
        fixture_handler: &FixtureHandler,
    ) -> Result<Self, PresetHandlerError> {
        let name = format!("Position Preset {}", id);

        let mut position = HashMap::new();
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
                    .position_pan_tilt()
                    .map_err(PresetHandlerError::FixtureError)?;

                let pan_tilt = fixture_position.to_pan_tilt(preset_handler, fixture_id);

                position.insert(fixture_id, pan_tilt);
            }
        }

        Ok(Self { id, name, position })
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

    pub fn position(&self, fixture_id: u32) -> Option<&[f32; 2]> {
        self.position.get(&fixture_id)
    }
}
