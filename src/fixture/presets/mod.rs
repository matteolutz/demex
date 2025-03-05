use std::{collections::HashMap, fmt::Debug};

use command_slice::CommandSlice;
use error::PresetHandlerError;
use group::FixtureGroup;
use mmacro::MMacro;
use preset::{FixturePreset, FixturePresetId};
use serde::{Deserialize, Serialize};

use crate::parser::nodes::{
    action::{
        functions::{record_function::RecordChannelTypeSelector, update_function::UpdateMode},
        Action,
    },
    fixture_selector::{FixtureSelector, FixtureSelectorContext},
};

use super::{
    channel2::{channel_type::FixtureChannelType, feature::feature_group::FeatureGroup},
    handler::{error::FixtureHandlerError, FixtureHandler},
    selection::FixtureSelection,
    sequence::{
        cue::{Cue, CueIdx, CueTiming, CueTrigger},
        Sequence,
    },
    timing::TimingHandler,
    Fixture,
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

    presets: HashMap<FixturePresetId, FixturePreset>,

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
        fixture_selection: FixtureSelection,
        id: u32,
        name: Option<String>,
    ) -> Result<(), PresetHandlerError> {
        if self.groups.contains_key(&id) {
            return Err(PresetHandlerError::PresetAlreadyExists(id));
        }

        let group = FixtureGroup::new(id, name, fixture_selection);
        self.groups.insert(id, group);
        Ok(())
    }

    pub fn get_group(&self, id: u32) -> Result<&FixtureGroup, PresetHandlerError> {
        self.groups
            .get(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))
    }

    pub fn get_group_mut(&mut self, id: u32) -> Result<&mut FixtureGroup, PresetHandlerError> {
        self.groups
            .get_mut(&id)
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

    pub fn delete_group(&mut self, id: u32) -> Result<(), PresetHandlerError> {
        self.groups
            .remove(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))?;
        Ok(())
    }
}

impl PresetHandler {
    pub fn record_preset(
        &mut self,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        id: FixturePresetId,
        name: Option<String>,
        fixture_handler: &FixtureHandler,
        timing_handler: &TimingHandler,
    ) -> Result<(), PresetHandlerError> {
        if self.presets.contains_key(&id) {
            return Err(PresetHandlerError::FeaturePresetAlreadyExists(id));
        }

        let feature_types = self
            .get_feature_group(id.feature_group_id)?
            .feature_types()
            .to_vec();

        let data = FixturePreset::generate_preset_data(
            fixture_handler,
            self,
            timing_handler,
            fixture_selector,
            fixture_selector_context,
            feature_types,
        )?;

        let preset =
            FixturePreset::new(id, name, self.get_feature_group(id.feature_group_id)?, data)?;

        self.presets.insert(id, preset);
        Ok(())
    }

    pub fn update_preset(
        &mut self,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        id: FixturePresetId,
        fixture_handler: &FixtureHandler,
        timing_handler: &TimingHandler,
        update_mode: UpdateMode,
    ) -> Result<usize, PresetHandlerError> {
        let preset = self.get_preset(id)?;
        let feature_types = self
            .get_feature_group(preset.id().feature_group_id)?
            .feature_types()
            .to_vec();

        let new_data = FixturePreset::generate_preset_data(
            fixture_handler,
            self,
            timing_handler,
            fixture_selector,
            fixture_selector_context,
            feature_types,
        )?;

        let preset = self.get_preset_mut(id)?;

        let values_updated = preset.update(new_data, update_mode)?;

        Ok(values_updated)
    }

    pub fn presets_mut(&mut self) -> &mut HashMap<FixturePresetId, FixturePreset> {
        &mut self.presets
    }

    pub fn presets(&self) -> &HashMap<FixturePresetId, FixturePreset> {
        &self.presets
    }

    pub fn presets_for_feature_group(&self, feature_group_id: u32) -> Vec<&FixturePreset> {
        self.presets
            .values()
            .filter(|p| p.id().feature_group_id == feature_group_id)
            .collect()
    }

    pub fn next_preset_id(&self, feature_group_id: u32) -> u32 {
        self.presets
            .keys()
            .filter(|k| k.feature_group_id == feature_group_id)
            .map(|k| k.preset_id)
            .max()
            .unwrap_or(0)
            + 1
    }

    pub fn rename_preset(
        &mut self,
        preset_id: FixturePresetId,
        new_name: String,
    ) -> Result<(), PresetHandlerError> {
        let preset = self.get_preset_mut(preset_id)?;
        *preset.name_mut() = new_name;
        Ok(())
    }

    pub fn get_preset_range(
        &self,
        preset_id_from: FixturePresetId,
        preset_id_to: FixturePresetId,
    ) -> Result<Vec<&FixturePreset>, PresetHandlerError> {
        let mut presets = Vec::new();

        if preset_id_from.feature_group_id != preset_id_to.feature_group_id {
            return Err(PresetHandlerError::FeatureGroupMismatch(
                preset_id_from.feature_group_id,
                preset_id_to.feature_group_id,
            ));
        }

        for i in preset_id_from.preset_id..=preset_id_to.preset_id {
            let preset = self.get_preset(FixturePresetId {
                feature_group_id: preset_id_from.feature_group_id,
                preset_id: i,
            })?;
            presets.push(preset);
        }

        Ok(presets)
    }

    pub fn get_preset(
        &self,
        preset_id: FixturePresetId,
    ) -> Result<&FixturePreset, PresetHandlerError> {
        self.presets
            .get(&preset_id)
            .ok_or(PresetHandlerError::FeaturePresetNotFound(preset_id))
    }

    pub fn get_preset_mut(
        &mut self,
        preset_id: FixturePresetId,
    ) -> Result<&mut FixturePreset, PresetHandlerError> {
        self.presets
            .get_mut(&preset_id)
            .ok_or(PresetHandlerError::FeaturePresetNotFound(preset_id))
    }

    pub fn get_preset_value_for_fixture(
        &self,
        preset_id: FixturePresetId,
        fixture: &Fixture,
        channel_type: FixtureChannelType,
        timing_handler: &TimingHandler,
    ) -> Option<u8> {
        let preset = self.get_preset(preset_id);

        if let Ok(preset) = preset {
            preset.value(fixture, channel_type, self, timing_handler)
        } else {
            None
        }
    }

    pub fn apply_preset(
        &mut self,
        preset_id: FixturePresetId,
        fixture_handler: &mut FixtureHandler,
        selection: FixtureSelection,
    ) -> Result<(), PresetHandlerError> {
        let feature_types = self
            .get_feature_group(preset_id.feature_group_id)?
            .feature_types()
            .to_vec();
        let preset = self.get_preset_mut(preset_id)?;

        for fixture_id in selection.fixtures() {
            preset.apply(
                fixture_handler.fixture(*fixture_id).ok_or(
                    PresetHandlerError::FixtureHandlerError(FixtureHandlerError::FixtureNotFound(
                        *fixture_id,
                    )),
                )?,
                &feature_types,
                selection.clone(),
            )?;
        }

        Ok(())
    }

    pub fn delete_preset(&mut self, preset_id: FixturePresetId) -> Result<(), PresetHandlerError> {
        self.presets
            .remove(&preset_id)
            .ok_or(PresetHandlerError::FeaturePresetNotFound(preset_id))?;
        Ok(())
    }

    pub fn stop_all(&mut self) {
        for preset in self.presets_mut().values_mut() {
            preset.stop();
        }
    }

    pub fn delete_preset_range(
        &mut self,
        preset_id_from: FixturePresetId,
        preset_id_to: FixturePresetId,
    ) -> Result<usize, PresetHandlerError> {
        if preset_id_from.feature_group_id != preset_id_to.feature_group_id {
            return Err(PresetHandlerError::FeatureGroupMismatch(
                preset_id_from.feature_group_id,
                preset_id_to.feature_group_id,
            ));
        }

        let mut count = 0;
        for id in preset_id_from.preset_id..=preset_id_to.preset_id {
            self.delete_preset(FixturePresetId {
                feature_group_id: preset_id_from.feature_group_id,
                preset_id: id,
            })?;
            count += 1;
        }

        Ok(count)
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

    pub fn get_macro(&self, id: u32) -> Result<&MMacro, PresetHandlerError> {
        self.macros
            .get(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))
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

    pub fn next_command_slice_id(&self) -> u32 {
        self.command_slices.keys().max().unwrap_or(&0) + 1
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

    pub fn update_sequence_cue(
        &mut self,
        sequence_id: u32,
        cue_idx: CueIdx,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        fixture_handler: &FixtureHandler,
        channel_type_selector: &RecordChannelTypeSelector,
        update_mode: UpdateMode,
    ) -> Result<usize, PresetHandlerError> {
        let selection = fixture_selector
            .get_selection(self, fixture_selector_context)
            .map_err(|err| PresetHandlerError::FixtureSelectorError(Box::new(err)))?;

        let sequence = self
            .sequences
            .get_mut(&sequence_id)
            .ok_or(PresetHandlerError::PresetNotFound(sequence_id))?;

        let cue = sequence
            .cues_mut()
            .iter_mut()
            .find(|c| c.cue_idx() == cue_idx)
            .ok_or(PresetHandlerError::CueNotFound(sequence_id, cue_idx))?;

        let cue_data = Cue::generate_cue_data(fixture_handler, &selection, channel_type_selector)?;

        let values_updated = cue.update(sequence_id, cue_data, &selection, update_mode)?;

        Ok(values_updated)
    }

    pub fn record_sequence_cue(
        &mut self,
        sequence_id: u32,
        fixture_handler: &FixtureHandler,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        cue_idx: Option<CueIdx>,
        channel_type_selector: &RecordChannelTypeSelector,
    ) -> Result<(), PresetHandlerError> {
        // does this cue already exist?
        if let Some(cue_idx) = cue_idx {
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
            Some(cue_idx) => cue_idx,
            None => {
                let sequence = self
                    .sequences
                    .get(&sequence_id)
                    .ok_or(PresetHandlerError::PresetNotFound(sequence_id))?;
                sequence.next_cue_idx()
            }
        };

        let selection = fixture_selector
            .get_selection(self, fixture_selector_context)
            .map_err(|err| PresetHandlerError::FixtureSelectorError(Box::new(err)))?;

        let cue_data = Cue::generate_cue_data(fixture_handler, &selection, channel_type_selector)?;

        let cue = Cue::new(
            discrete_cue_idx,
            cue_data,
            selection,
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

    pub fn rename_sequence_cue(
        &mut self,
        sequence_id: u32,
        cue_idx: CueIdx,
        new_name: String,
    ) -> Result<(), PresetHandlerError> {
        let sequence = self
            .sequences
            .get_mut(&sequence_id)
            .ok_or(PresetHandlerError::PresetNotFound(sequence_id))?;

        let cue = sequence
            .cues_mut()
            .iter_mut()
            .find(|c| c.cue_idx() == cue_idx)
            .ok_or(PresetHandlerError::CueNotFound(sequence_id, cue_idx))?;

        *cue.name_mut() = new_name;
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

    pub fn delete_sequence(&mut self, id: u32) -> Result<(), PresetHandlerError> {
        self.sequences
            .remove(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))?;
        Ok(())
    }
}

// Feature groupus
impl PresetHandler {
    pub fn feature_groups(&self) -> &HashMap<u32, FeatureGroup> {
        &self.feature_groups
    }

    pub fn feature_groups_mut(&mut self) -> &mut HashMap<u32, FeatureGroup> {
        &mut self.feature_groups
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
