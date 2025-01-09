use std::collections::HashMap;

use color::FixtureColorPreset;
use command_slice::CommandSlice;
use error::PresetHandlerError;
use group::FixtureGroup;
use mmacro::MMacro;
use position::FixturePositionPreset;

use crate::parser::nodes::{
    action::{result::ActionRunResult, Action},
    fixture_selector::{FixtureSelector, FixtureSelectorContext},
};

use super::{handler::FixtureHandler, sequence::Sequence};

pub mod color;
pub mod command_slice;
pub mod error;
pub mod group;
pub mod mmacro;
pub mod position;

pub struct PresetHandler {
    groups: HashMap<u32, FixtureGroup>,
    colors: HashMap<u32, FixtureColorPreset>,
    positions: HashMap<u32, FixturePositionPreset>,
    sequences: HashMap<u32, Sequence>,
    macros: HashMap<u32, MMacro>,
    command_slices: HashMap<u32, CommandSlice>,
}

impl PresetHandler {
    pub fn new() -> Self {
        PresetHandler {
            groups: HashMap::new(),
            colors: HashMap::new(),
            positions: HashMap::new(),
            sequences: HashMap::new(),
            macros: HashMap::new(),
            command_slices: HashMap::new(),
        }
    }
}

// Groups
impl PresetHandler {
    pub fn record_group(
        &mut self,
        fixture_selector: FixtureSelector,
        id: u32,
    ) -> Result<(), PresetHandlerError> {
        if self.groups.contains_key(&id) {
            return Err(PresetHandlerError::PresetAlreadyExists(id));
        }

        let group = FixtureGroup::new(id, fixture_selector);
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

// Colors
impl PresetHandler {
    pub fn record_color(
        &mut self,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        id: u32,
        fixture_handler: &FixtureHandler,
    ) -> Result<(), PresetHandlerError> {
        if self.colors.contains_key(&id) {
            return Err(PresetHandlerError::PresetAlreadyExists(id));
        }

        let color = FixtureColorPreset::new(
            id,
            fixture_selector,
            fixture_selector_context,
            self,
            fixture_handler,
        )?;
        self.colors.insert(id, color);
        Ok(())
    }

    pub fn get_color(&self, id: u32) -> Result<&FixtureColorPreset, PresetHandlerError> {
        self.colors
            .get(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))
    }

    pub fn get_color_for_fixture(&self, preset_id: u32, fixture_id: u32) -> Option<[f32; 4]> {
        let color = self.get_color(preset_id);

        if let Ok(color) = color {
            color.color(fixture_id).copied()
        } else {
            None
        }
    }

    pub fn rename_color(&mut self, id: u32, new_name: String) -> Result<(), PresetHandlerError> {
        let color = self
            .colors
            .get_mut(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))?;
        *color.name_mut() = new_name;
        Ok(())
    }

    pub fn colors(&self) -> &HashMap<u32, FixtureColorPreset> {
        &self.colors
    }
}

// Positions
impl PresetHandler {
    pub fn record_position(
        &mut self,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        id: u32,
        fixture_handler: &FixtureHandler,
    ) -> Result<(), PresetHandlerError> {
        if self.positions.contains_key(&id) {
            return Err(PresetHandlerError::PresetAlreadyExists(id));
        }

        let position = FixturePositionPreset::new(
            id,
            fixture_selector,
            fixture_selector_context,
            self,
            fixture_handler,
        )?;
        self.positions.insert(id, position);
        Ok(())
    }

    pub fn get_position(&self, id: u32) -> Result<&FixturePositionPreset, PresetHandlerError> {
        self.positions
            .get(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))
    }

    pub fn get_position_for_fixture(&self, preset_id: u32, fixture_id: u32) -> Option<[f32; 2]> {
        let pos = self.get_position(preset_id);

        if let Ok(pos) = pos {
            pos.position(fixture_id).copied()
        } else {
            None
        }
    }

    pub fn rename_position(&mut self, id: u32, new_name: String) -> Result<(), PresetHandlerError> {
        let position = self
            .positions
            .get_mut(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))?;
        *position.name_mut() = new_name;
        Ok(())
    }

    pub fn positions(&self) -> &HashMap<u32, FixturePositionPreset> {
        &self.positions
    }
}

// Sequences
impl PresetHandler {
    pub fn add_sequence(&mut self, sequence: Sequence) {
        self.sequences.insert(sequence.id(), sequence);
    }

    pub fn sequence(&self, id: u32) -> Option<&Sequence> {
        self.sequences.get(&id)
    }
}

// Macros
impl PresetHandler {
    pub fn record_macro(&mut self, id: u32, action: Box<Action>) -> Result<(), PresetHandlerError> {
        if self.macros.contains_key(&id) {
            return Err(PresetHandlerError::PresetAlreadyExists(id));
        }

        self.macros.insert(id, MMacro::new(id, action));
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

    pub fn run_macro(
        &mut self,
        id: u32,
        fixture_handler: &mut FixtureHandler,
        fixture_selector_context: FixtureSelectorContext,
    ) -> Result<ActionRunResult, PresetHandlerError> {
        let mmacro = self
            .macros
            .get(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))?
            .clone();

        mmacro
            .run(fixture_handler, self, fixture_selector_context)
            .map_err(|err| PresetHandlerError::MacroExecutionError(Box::new(err)))
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
