use std::collections::HashMap;

use command_slice::CommandSlice;
use error::PresetHandlerError;
use group::FixtureGroup;
use mmacro::MMacro;
use preset::FixturePreset;
use serde::{Deserialize, Serialize};

use crate::parser::nodes::{
    action::{
        Action, ChannelTypeSelectorActionData, CueIdxSelectorActionData, UpdateModeActionData,
    },
    fixture_selector::{FixtureSelector, FixtureSelectorContext},
};

use super::{
    channel::value::FixtureChannelDiscreteValue,
    feature::group::FeatureGroup,
    handler::FixtureHandler,
    sequence::{
        cue::{Cue, CueTiming, CueTrigger},
        Sequence,
    },
    updatables::UpdatableHandler,
};

pub mod command_slice;
pub mod error;
pub mod group;
pub mod mmacro;
pub mod preset;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PresetHandler {
    groups: HashMap<u32, FixtureGroup>,

    macros: HashMap<u32, MMacro>,
    command_slices: HashMap<u32, CommandSlice>,

    sequences: HashMap<u32, Sequence>,

    presets: HashMap<u32, FixturePreset>,

    feature_groups: HashMap<u32, FeatureGroup>,
}

impl Default for PresetHandler {
    fn default() -> Self {
        Self {
            feature_groups: FeatureGroup::default_feature_groups(),
            groups: HashMap::new(),
            macros: HashMap::new(),
            command_slices: HashMap::new(),
            sequences: HashMap::new(),
            presets: HashMap::new(),
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

    pub fn next_group_id(&self) -> u32 {
        self.groups.keys().max().unwrap_or(&0) + 1
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
        feature_group_id: u32,
        updatable_handler: &UpdatableHandler,
    ) -> Result<(), PresetHandlerError> {
        if self.presets.contains_key(&id) {
            return Err(PresetHandlerError::PresetAlreadyExists(id));
        }

        let preset = FixturePreset::new(
            id,
            name,
            fixture_selector,
            fixture_selector_context,
            feature_group_id,
            self,
            fixture_handler,
            updatable_handler,
        )?;

        self.presets.insert(id, preset);
        Ok(())
    }

    pub fn update_preset(
        &mut self,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        id: u32,
        fixture_handler: &FixtureHandler,
        updatable_handler: &UpdatableHandler,
        update_mode: &UpdateModeActionData,
    ) -> Result<usize, PresetHandlerError> {
        let preset = self.get_preset(id)?;
        let feature_group = self.get_feature_group(preset.feature_group_id())?;

        let mut new_data: HashMap<u32, HashMap<u16, FixtureChannelDiscreteValue>> = HashMap::new();

        for fixture_id in fixture_selector
            .get_fixtures(self, fixture_selector_context)
            .map_err(|err| PresetHandlerError::FixtureSelectorError(Box::new(err)))?
        {
            let fixture = fixture_handler.fixture_immut(fixture_id);
            if let Some(fixture) = fixture {
                let mut new_values = HashMap::new();

                for channel_type in feature_group.channel_types() {
                    if !fixture.channel_types().contains(&channel_type) {
                        continue;
                    }

                    let fixture_channel_value = fixture
                        .channel_value(*channel_type, self, updatable_handler)
                        .map_err(PresetHandlerError::FixtureError)?;

                    new_values.insert(*channel_type, fixture_channel_value.to_discrete());
                }

                new_data.insert(fixture_id, new_values);
            }
        }

        let preset = self.get_preset_mut(id)?;

        let values_updated = preset.update(new_data, update_mode)?;

        Ok(values_updated)
    }

    pub fn presets_mut(&mut self) -> &mut HashMap<u32, FixturePreset> {
        &mut self.presets
    }

    pub fn presets(&self) -> &HashMap<u32, FixturePreset> {
        &self.presets
    }

    pub fn presets_for_feature_group(&self, feature_group_id: u32) -> Vec<&FixturePreset> {
        self.presets
            .values()
            .filter(|p| p.feature_group_id() == feature_group_id)
            .collect()
    }

    pub fn next_preset_id(&self) -> u32 {
        self.presets.keys().max().unwrap_or(&0) + 1
    }

    pub fn rename_preset(
        &mut self,
        preset_id: u32,
        new_name: String,
    ) -> Result<(), PresetHandlerError> {
        let preset = self.get_preset_mut(preset_id)?;
        *preset.name_mut() = new_name;
        Ok(())
    }

    pub fn get_preset_range(
        &self,
        preset_id_from: u32,
        preset_id_to: u32,
    ) -> Result<Vec<&FixturePreset>, PresetHandlerError> {
        let mut presets = Vec::new();

        for i in preset_id_from..=preset_id_to {
            let preset = self.get_preset(i)?;
            presets.push(preset);
        }

        Ok(presets)
    }

    pub fn get_preset(&self, preset_id: u32) -> Result<&FixturePreset, PresetHandlerError> {
        self.presets
            .get(&preset_id)
            .ok_or(PresetHandlerError::PresetNotFound(preset_id))
    }

    pub fn get_preset_mut(
        &mut self,
        preset_id: u32,
    ) -> Result<&mut FixturePreset, PresetHandlerError> {
        self.presets
            .get_mut(&preset_id)
            .ok_or(PresetHandlerError::PresetNotFound(preset_id))
    }

    pub fn get_preset_for_fixture(
        &self,
        preset_id: u32,
        fixture_id: u32,
        channel_type: u16,
    ) -> Option<FixtureChannelDiscreteValue> {
        let preset = self.get_preset(preset_id);

        if let Ok(preset) = preset {
            preset.value(fixture_id, channel_type).cloned()
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

    pub fn next_macro_id(&self) -> u32 {
        self.macros.keys().max().unwrap_or(&0) + 1
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

    pub fn next_sequence_id(&self) -> u32 {
        self.sequences.keys().max().unwrap_or(&0) + 1
    }

    pub fn record_sequence_cue(
        &mut self,
        sequence_id: u32,
        fixture_handler: &FixtureHandler,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        cue_idx: CueIdxSelectorActionData,
        channel_type_selector: &ChannelTypeSelectorActionData,
    ) -> Result<(), PresetHandlerError> {
        // does this cue already exist?
        if let CueIdxSelectorActionData::Discrete(cue_idx) = cue_idx {
            if self
                .sequences
                .get(&sequence_id)
                .ok_or(PresetHandlerError::PresetNotFound(sequence_id))?
                .cues()
                .iter()
                .any(|c| c.cue_idx() == cue_idx)
            {
                return Err(PresetHandlerError::CueAlreadyExists(sequence_id, cue_idx));
            }
        }

        let discrete_cue_idx = match cue_idx {
            CueIdxSelectorActionData::Discrete(cue_idx) => cue_idx,
            CueIdxSelectorActionData::Next => {
                let cues = self
                    .sequences
                    .get(&sequence_id)
                    .ok_or(PresetHandlerError::PresetNotFound(sequence_id))?
                    .cues();

                if cues.is_empty() {
                    (1, 0)
                } else {
                    (cues.last().unwrap().cue_idx().0 + 1, 0)
                }
            }
        };

        let mut cue_data = HashMap::new();

        for fixture_id in fixture_selector
            .get_fixtures(self, fixture_selector_context)
            .map_err(|err| PresetHandlerError::FixtureSelectorError(Box::new(err)))?
        {
            if let Some(fixture) = fixture_handler.fixture_immut(fixture_id) {
                cue_data.insert(
                    fixture_id,
                    channel_type_selector
                        .get_channel_values(fixture)
                        .map_err(PresetHandlerError::FixtureError)?,
                );
            }
        }

        let cue = Cue::new(
            discrete_cue_idx,
            cue_data,
            0.0,
            None,
            0.0,
            None,
            0.0,
            CueTiming::default(),
            CueTrigger::Manual,
        );

        let cues = self
            .sequences
            .get_mut(&sequence_id)
            .ok_or(PresetHandlerError::PresetNotFound(sequence_id))?
            .cues_mut();

        for (idx, c) in cues.iter().enumerate() {
            if c.cue_idx() > discrete_cue_idx {
                cues.insert(idx, cue);
                return Ok(());
            }
        }

        // if we didn't insert the cue yet, it means it's the last cue
        cues.push(cue);

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

    pub fn get_sequence_mut(&mut self, id: u32) -> Result<&mut Sequence, PresetHandlerError> {
        self.sequences
            .get_mut(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))
    }

    pub fn sequences(&self) -> &HashMap<u32, Sequence> {
        &self.sequences
    }
}

// Feature groupus
impl PresetHandler {
    pub fn feature_groups(&self) -> &HashMap<u32, FeatureGroup> {
        &self.feature_groups
    }

    pub fn get_feature_group(&self, id: u32) -> Result<&FeatureGroup, PresetHandlerError> {
        self.feature_groups
            .get(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))
    }

    pub fn get_feature_group_mut(
        &mut self,
        id: u32,
    ) -> Result<&mut FeatureGroup, PresetHandlerError> {
        self.feature_groups
            .get_mut(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))
    }

    pub fn rename_feature_group(
        &mut self,
        id: u32,
        new_name: String,
    ) -> Result<(), PresetHandlerError> {
        let group = self
            .feature_groups
            .get_mut(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))?;
        *group.name_mut() = new_name;
        Ok(())
    }
}
