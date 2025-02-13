use std::collections::HashMap;

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::{
    fixture::{channel2::channel_type::FixtureChannelType, handler::FixtureHandler},
    parser::nodes::{
        action::UpdateModeActionData,
        fixture_selector::{FixtureSelector, FixtureSelectorContext},
    },
};

use super::{error::PresetHandlerError, PresetHandler};

#[derive(Debug, Serialize, Deserialize, Clone, EguiProbe)]
pub struct FixturePreset {
    #[egui_probe(skip)]
    id: u32,

    name: String,

    #[egui_probe(skip)]
    feature_group_id: u32,

    #[egui_probe(skip)]
    data: HashMap<u32, HashMap<FixtureChannelType, u8>>,
}

impl FixturePreset {
    pub fn new(
        id: u32,
        name: Option<String>,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        feature_group_id: u32,
        preset_handler: &PresetHandler,
        fixture_handler: &FixtureHandler,
    ) -> Result<Self, PresetHandlerError> {
        let feature_group = preset_handler.get_feature_group(feature_group_id)?;

        let name = name.unwrap_or(format!("{} Preset {}", feature_group.name(), id));

        let mut data = HashMap::new();

        for fixture_id in fixture_selector
            .get_fixtures(preset_handler, fixture_selector_context)
            .map_err(|err| PresetHandlerError::FixtureSelectorError(Box::new(err)))?
        {
            let fixture = fixture_handler.fixture_immut(fixture_id);

            if let Some(fixture) = fixture {
                let mut values = HashMap::new();

                for channel_type in feature_group.channel_types() {
                    if !fixture.channel_types().contains(&channel_type) {
                        continue;
                    }

                    let fixture_channel_value = fixture
                        .channel_value_programmer(*channel_type)
                        .map_err(PresetHandlerError::FixtureError)?;

                    if fixture_channel_value.is_home() {
                        continue;
                    }

                    if let Ok(discrete_value) = fixture_channel_value.to_discrete_value(
                        fixture_id,
                        *channel_type,
                        preset_handler,
                    ) {
                        values.insert(*channel_type, discrete_value);
                    }
                }

                data.insert(fixture_id, values);
            }
        }

        Ok(Self {
            id,
            name,
            feature_group_id,
            data,
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

    pub fn feature_group_id(&self) -> u32 {
        self.feature_group_id
    }

    pub fn channel_types(&self, preset_handler: &PresetHandler) -> Vec<FixtureChannelType> {
        preset_handler
            .get_feature_group(self.feature_group_id)
            .unwrap()
            .channel_types()
            .to_vec()
    }

    pub fn value(&self, fixture_id: u32, channel_type: FixtureChannelType) -> Option<u8> {
        self.data
            .get(&fixture_id)
            .and_then(|values| values.get(&channel_type).copied())
    }

    pub fn update(
        &mut self,
        values_to_update: HashMap<u32, HashMap<FixtureChannelType, u8>>,
        update_mode: &UpdateModeActionData,
    ) -> Result<usize, PresetHandlerError> {
        let mut updated = 0;

        for (fixture_id, new_fixture_values) in values_to_update {
            // if we already have a value for this fixture and we are not in override mode, skip
            if self.data.contains_key(&fixture_id) && update_mode != &UpdateModeActionData::Override
            {
                continue;
            }

            // Insert or update
            self.data.insert(fixture_id, new_fixture_values);

            updated += 1;
        }

        Ok(updated)
    }
}
