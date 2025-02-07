use std::collections::HashMap;

use command_slice::CommandSlice;
use error::PresetHandlerError;
use group::FixtureGroup;
use mmacro::MMacro;
use preset::FixturePreset;
use serde::{Deserialize, Serialize};

use crate::parser::nodes::{
    action::{Action, UpdateModeActionData},
    fixture_selector::{FixtureSelector, FixtureSelectorContext},
};

use super::{
    channel::{
        value::FixtureChannelDiscreteValue, FIXTURE_CHANNEL_COLOR_ID, FIXTURE_CHANNEL_INTENSITY_ID,
        FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
    },
    handler::FixtureHandler,
    sequence::Sequence,
    updatables::UpdatableHandler,
};

pub mod command_slice;
pub mod error;
pub mod group;
pub mod mmacro;
pub mod preset;

#[derive(Serialize, Deserialize, Clone)]
pub struct PresetHandler {
    groups: HashMap<u32, FixtureGroup>,

    dimmers: HashMap<u32, FixturePreset>,
    colors: HashMap<u32, FixturePreset>,
    positions: HashMap<u32, FixturePreset>,

    macros: HashMap<u32, MMacro>,
    command_slices: HashMap<u32, CommandSlice>,

    sequences: HashMap<u32, Sequence>,
}

impl PresetHandler {
    pub fn new() -> Self {
        PresetHandler {
            groups: HashMap::new(),
            dimmers: HashMap::new(),
            colors: HashMap::new(),
            positions: HashMap::new(),
            macros: HashMap::new(),
            command_slices: HashMap::new(),
            sequences: HashMap::new(),
        }
    }
}

// Groups
impl PresetHandler {
    pub fn record_group(
        &mut self,
        fixture_selector: FixtureSelector,
        id: u32,
        name: Option<String>,
    ) -> Result<(), PresetHandlerError> {
        if self.groups.contains_key(&id) {
            return Err(PresetHandlerError::PresetAlreadyExists(id));
        }

        let group = FixtureGroup::new(id, name, fixture_selector);
        self.groups.insert(id, group);
        Ok(())
    }

    pub fn get_group(&self, id: u32) -> Result<&FixtureGroup, PresetHandlerError> {
        self.groups
            .get(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))
    }

    pub fn rename_group(&mut self, id: u32, new_name: String) -> Result<(), PresetHandlerError> {
        let group = self
            .groups
            .get_mut(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))?;
        *group.name_mut() = new_name;
        Ok(())
    }

    pub fn groups(&self) -> &HashMap<u32, FixtureGroup> {
        &self.groups
    }
}

impl PresetHandler {
    pub fn record_preset(
        &mut self,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        id: u32,
        name: Option<String>,
        fixture_handler: &FixtureHandler,
        channel_type: u16,
        updatable_handler: &UpdatableHandler,
    ) -> Result<(), PresetHandlerError> {
        if self.presets_mut(channel_type).contains_key(&id) {
            return Err(PresetHandlerError::PresetAlreadyExists(id));
        }

        let preset = FixturePreset::new(
            id,
            name,
            fixture_selector,
            fixture_selector_context,
            channel_type,
            self,
            fixture_handler,
            updatable_handler,
        )?;

        self.presets_mut(channel_type).insert(id, preset);
        Ok(())
    }

    pub fn update_preset(
        &mut self,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        channel_type: u16,
        id: u32,
        fixture_handler: &FixtureHandler,
        updatable_handler: &UpdatableHandler,
        update_mode: &UpdateModeActionData,
    ) -> Result<usize, PresetHandlerError> {
        let mut new_values: HashMap<u32, FixtureChannelDiscreteValue> = HashMap::new();

        for fixture_id in fixture_selector
            .get_fixtures(self, fixture_selector_context)
            .map_err(|err| PresetHandlerError::FixtureSelectorError(Box::new(err)))?
        {
            let fixture = fixture_handler.fixture_immut(fixture_id);
            if let Some(fixture) = fixture {
                if !fixture.channel_types().contains(&channel_type) {
                    continue;
                }

                let fixture_channel_value = fixture
                    .channel_value(channel_type, self, updatable_handler)
                    .map_err(PresetHandlerError::FixtureError)?;

                new_values.insert(fixture_id, fixture_channel_value.to_discrete());
            }
        }

        let preset = self.get_preset_mut(id, channel_type)?;

        let values_updated = preset.update(new_values, update_mode)?;

        Ok(values_updated)
    }

    pub fn presets_mut(&mut self, channel_type: u16) -> &mut HashMap<u32, FixturePreset> {
        match channel_type {
            FIXTURE_CHANNEL_INTENSITY_ID => &mut self.dimmers,
            FIXTURE_CHANNEL_COLOR_ID => &mut self.colors,
            FIXTURE_CHANNEL_POSITION_PAN_TILT_ID => &mut self.positions,
            _ => todo!("not implemented"),
        }
    }

    pub fn presets(&self, channel_type: u16) -> &HashMap<u32, FixturePreset> {
        match channel_type {
            FIXTURE_CHANNEL_INTENSITY_ID => &self.dimmers,
            FIXTURE_CHANNEL_COLOR_ID => &self.colors,
            FIXTURE_CHANNEL_POSITION_PAN_TILT_ID => &self.positions,
            _ => todo!("not implemented"),
        }
    }

    pub fn rename_preset(
        &mut self,
        preset_id: u32,
        channel_type: u16,
        new_name: String,
    ) -> Result<(), PresetHandlerError> {
        let preset = self.get_preset_mut(preset_id, channel_type)?;
        *preset.name_mut() = new_name;
        Ok(())
    }

    pub fn get_preset(
        &self,
        preset_id: u32,
        channel_type: u16,
    ) -> Result<&FixturePreset, PresetHandlerError> {
        self.presets(channel_type)
            .get(&preset_id)
            .ok_or(PresetHandlerError::PresetNotFound(preset_id))
    }

    pub fn get_preset_mut(
        &mut self,
        preset_id: u32,
        channel_type: u16,
    ) -> Result<&mut FixturePreset, PresetHandlerError> {
        self.presets_mut(channel_type)
            .get_mut(&preset_id)
            .ok_or(PresetHandlerError::PresetNotFound(preset_id))
    }

    pub fn get_preset_for_fixture(
        &self,
        preset_id: u32,
        channel_type: u16,
        fixture_id: u32,
    ) -> Option<FixtureChannelDiscreteValue> {
        let preset = self.get_preset(preset_id, channel_type);

        if let Ok(preset) = preset {
            preset.value(fixture_id).cloned()
        } else {
            None
        }
    }
}

// Macros
impl PresetHandler {
    pub fn create_macro(
        &mut self,
        id: u32,
        name: Option<String>,
        action: Box<Action>,
    ) -> Result<(), PresetHandlerError> {
        if self.macros.contains_key(&id) {
            return Err(PresetHandlerError::PresetAlreadyExists(id));
        }

        self.macros.insert(id, MMacro::new(id, name, action));
        Ok(())
    }

    pub fn rename_macro(&mut self, id: u32, new_name: String) -> Result<(), PresetHandlerError> {
        let mmacro = self
            .macros
            .get_mut(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))?;
        *mmacro.name_mut() = new_name;
        Ok(())
    }

    pub fn delete_macro(&mut self, id: u32) -> Result<(), PresetHandlerError> {
        self.macros
            .remove(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))?;
        Ok(())
    }

    pub fn macros(&self) -> &HashMap<u32, MMacro> {
        &self.macros
    }
}

// Command Slices
impl PresetHandler {
    pub fn record_command_slice(&mut self, slice: CommandSlice) -> Result<(), PresetHandlerError> {
        if self.command_slices.contains_key(&slice.id()) {
            return Err(PresetHandlerError::PresetAlreadyExists(slice.id()));
        }

        self.command_slices.insert(slice.id(), slice);
        Ok(())
    }

    pub fn rename_command_slice(
        &mut self,
        id: u32,
        new_name: String,
    ) -> Result<(), PresetHandlerError> {
        let slice = self
            .command_slices
            .get_mut(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))?;
        *slice.name_mut() = new_name;
        Ok(())
    }

    pub fn get_command_slice(&self, id: u32) -> Result<&CommandSlice, PresetHandlerError> {
        self.command_slices
            .get(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))
    }

    pub fn command_slices(&self) -> &HashMap<u32, CommandSlice> {
        &self.command_slices
    }
}

// Sequences
impl PresetHandler {
    pub fn create_sequence(
        &mut self,
        id: u32,
        name: Option<String>,
    ) -> Result<(), PresetHandlerError> {
        if self.sequences.contains_key(&id) {
            return Err(PresetHandlerError::PresetAlreadyExists(id));
        }

        self.sequences.insert(
            id,
            Sequence::new(id, name.unwrap_or(format!("Sequence {}", id))),
        );
        Ok(())
    }

    pub fn rename_sequence(&mut self, id: u32, new_name: String) -> Result<(), PresetHandlerError> {
        let sequence = self
            .sequences
            .get_mut(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))?;
        *sequence.name_mut() = new_name;
        Ok(())
    }

    pub fn get_sequence(&self, id: u32) -> Result<&Sequence, PresetHandlerError> {
        self.sequences
            .get(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))
    }

    pub fn sequences(&self) -> &HashMap<u32, Sequence> {
        &self.sequences
    }
}
