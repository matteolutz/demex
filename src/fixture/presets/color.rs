use std::collections::HashMap;

use crate::{
    fixture::{
        channel::{color::FixtureColorValue, FIXTURE_CHANNEL_COLOR_ID},
        handler::FixtureHandler,
    },
    parser::nodes::fixture_selector::FixtureSelector,
};

use super::{error::PresetHandlerError, PresetHandler};

#[derive(Debug)]
pub struct FixtureColorPreset {
    id: u32,
    name: String,
    color: HashMap<u32, [f32; 4]>,
}

impl FixtureColorPreset {
    pub fn new(
        id: u32,
        fixture_selector: &FixtureSelector,
        preset_handler: &PresetHandler,
        fixture_handler: &FixtureHandler,
    ) -> Result<Self, PresetHandlerError> {
        let name = format!("Color Preset {}", id);

        let mut color = HashMap::new();
        for fixture_id in fixture_selector
            .get_fixtures(preset_handler)
            .map_err(|err| PresetHandlerError::FixtureSelectorError(Box::new(err)))?
        {
            let fixture = fixture_handler.fixture_immut(fixture_id);
            if let Some(fixture) = fixture {
                if !fixture.channel_types().contains(&FIXTURE_CHANNEL_COLOR_ID) {
                    continue;
                }

                let fixture_color = fixture.color().map_err(PresetHandlerError::FixtureError)?;

                let rgbw = match fixture_color {
                    FixtureColorValue::Rgbw(rgbw) => rgbw,
                    FixtureColorValue::Preset(preset_id) => preset_handler
                        .get_color_for_fixture(preset_id, fixture_id)
                        .unwrap_or([0.0, 0.0, 0.0, 0.0]),
                };

                color.insert(fixture_id, rgbw);
            }
        }

        Ok(Self { id, name, color })
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

    pub fn color(&self, fixture_id: u32) -> Option<&[f32; 4]> {
        self.color.get(&fixture_id)
    }
}
