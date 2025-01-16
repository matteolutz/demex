use std::collections::HashMap;

use command_slice::CommandSlice;
use error::PresetHandlerError;
use group::FixtureGroup;
use mmacro::MMacro;
use preset::FixturePreset;
use serde::{Deserialize, Serialize};

use crate::parser::nodes::{
    action::{result::ActionRunResult, Action},
    fixture_selector::{FixtureSelector, FixtureSelectorContext},
};

use super::{
    channel::{
        value::FixtureChannelDiscreteValue, FIXTURE_CHANNEL_COLOR_ID,
        FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
    },
    handler::FixtureHandler,
    sequence::runtime::SequenceRuntime,
};

pub mod command_slice;
pub mod error;
pub mod group;
pub mod mmacro;
pub mod preset;

#[derive(Serialize, Deserialize)]
pub struct PresetHandler {
    groups: HashMap<u32, FixtureGroup>,

    colors: HashMap<u32, FixturePreset>,
    positions: HashMap<u32, FixturePreset>,

    sequence_runtimes: HashMap<u32, SequenceRuntime>,
    macros: HashMap<u32, MMacro>,
    command_slices: HashMap<u32, CommandSlice>,
}

impl PresetHandler {
    pub fn new() -> Self {
        PresetHandler {
            groups: HashMap::new(),
            colors: HashMap::new(),
            positions: HashMap::new(),
            sequence_runtimes: HashMap::new(),
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

impl PresetHandler {
    pub fn record_preset(
        &mut self,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        id: u32,
        fixture_handler: &FixtureHandler,
        channel_type: u16,
    ) -> Result<(), PresetHandlerError> {
        if self.presets_mut(channel_type).contains_key(&id) {
            return Err(PresetHandlerError::PresetAlreadyExists(id));
        }

        let preset = FixturePreset::new(
            id,
            fixture_selector,
            fixture_selector_context,
            channel_type,
            self,
            fixture_handler,
        )?;

        self.presets_mut(channel_type).insert(id, preset);
        Ok(())
    }

    pub fn presets_mut(&mut self, channel_type: u16) -> &mut HashMap<u32, FixturePreset> {
        match channel_type {
            FIXTURE_CHANNEL_COLOR_ID => &mut self.colors,
            FIXTURE_CHANNEL_POSITION_PAN_TILT_ID => &mut self.positions,
            _ => todo!("not implemented"),
        }
    }

    pub fn presets(&self, channel_type: u16) -> &HashMap<u32, FixturePreset> {
        match channel_type {
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

// Sequence Runtimes
impl PresetHandler {
    pub fn add_sequence_runtime(&mut self, runtime: SequenceRuntime) {
        self.sequence_runtimes.insert(runtime.id(), runtime);
    }

    pub fn sequence_runtime(&self, id: u32) -> Option<&SequenceRuntime> {
        self.sequence_runtimes.get(&id)
    }

    pub fn sequence_runtime_mut(&mut self, id: u32) -> Option<&mut SequenceRuntime> {
        self.sequence_runtimes.get_mut(&id)
    }

    pub fn sequence_runtimes(&self) -> &HashMap<u32, SequenceRuntime> {
        &self.sequence_runtimes
    }

    pub fn sequence_runtime_keys(&self) -> Vec<u32> {
        self.sequence_runtimes.keys().cloned().collect()
    }

    pub fn update_sequence_runtimes(
        &mut self,
        delta_time: f64,
        fixture_handler: &mut FixtureHandler,
    ) {
        for (_, runtime) in self.sequence_runtimes.iter_mut() {
            runtime.update(delta_time, fixture_handler);
        }
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
