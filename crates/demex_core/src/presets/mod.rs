use std::{collections::HashMap, fmt::Debug};

use command_slice::CommandSlice;
use error::PresetHandlerError;
use group::FixtureGroup;
use itertools::Itertools;
use mmacro::MMacro;
use preset::{FixturePreset, FixturePresetData, FixturePresetId};
use serde::{Deserialize, Serialize};

use crate::{
    channel3::{attribute::FixtureChannel3Attribute, utils::HashMapExt},
    command::parser::nodes::{
        action::{
            Action,
            functions::{record_function::RecordChannelTypeSelector, update_function::UpdateMode},
        },
        fixture_selector::{FixtureSelector, FixtureSelectorContext},
    },
    engine::component::Component,
    event::{DemexEvent, list::DemexEventList},
    fixture::Fixture,
    patch::Patch,
    pool::{Pool, PoolError, PoolType},
    state::fixture_state_handler::FixtureStateHandler,
};

use super::{
    channel3::{
        channel_value::{FixtureChannelValue2PresetState, FixtureChannelValue3},
        feature::feature_group::FixtureChannel3FeatureGroup,
    },
    effect::feature::runtime::FeatureEffectRuntime,
    effect2::effect::Effect2,
    selection::FixtureSelection,
    sequence::{
        Sequence,
        cue::{Cue, CueIdx, CueTiming, CueTrigger},
    },
    timing::TimingHandler,
};

pub mod command_slice;
pub mod error;
pub mod group;
pub mod mmacro;
pub mod preset;

impl Component for PresetHandler {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PresetHandler {
    groups: HashMap<u32, FixtureGroup>,

    macros: HashMap<u32, MMacro>,
    command_slices: HashMap<u32, CommandSlice>,

    sequences: HashMap<u32, Sequence>,

    presets: HashMap<FixturePresetId, FixturePreset>,
}

impl Default for PresetHandler {
    fn default() -> Self {
        Self {
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
        event_list: &mut DemexEventList,
    ) -> Result<(), PresetHandlerError> {
        if self.groups.contains_key(&id) {
            return Err(PresetHandlerError::PresetAlreadyExists(id));
        }

        let group = FixtureGroup::new(id, name, fixture_selection);
        self.groups.insert(id, group);
        event_list.push(DemexEvent::PoolItemAdded(PoolType::Group, id));

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

    pub fn groups(&self) -> &HashMap<u32, FixtureGroup> {
        &self.groups
    }

    pub fn next_group_id(&self) -> u32 {
        self.groups.keys().max().unwrap_or(&0) + 1
    }

    pub fn delete_group(
        &mut self,
        id: u32,
        event_list: &mut DemexEventList,
    ) -> Result<(), PresetHandlerError> {
        self.groups
            .remove(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))?;
        event_list.push(DemexEvent::PoolItemsDeleted {
            pool_type: PoolType::Group,
            from_id: id,
            to_id: id,
        });

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
        should_next: bool,
        patch: &Patch,
        fixture_handler: &FixtureStateHandler,
        timing_handler: &TimingHandler,
        event_list: &mut DemexEventList,
    ) -> Result<(), PresetHandlerError> {
        let discrete_data = FixturePreset::generate_preset_data(
            patch,
            fixture_handler,
            self,
            timing_handler,
            fixture_selector,
            fixture_selector_context,
            id.feature_group,
        )?;

        if let Some(preset) = self.presets.get_mut(&id) {
            if should_next {
                preset.record_next(discrete_data, patch, event_list)?;
                return Ok(());
            } else {
                return Err(PresetHandlerError::FeaturePresetAlreadyExists(id));
            }
        }

        let display_colors = if id.feature_group == FixtureChannel3FeatureGroup::Color {
            discrete_data
                .iter()
                .filter_map(|(f_path, value)| {
                    patch
                        .fixture(f_path)
                        .ok()
                        .and_then(|fixture| value.get_color(fixture))
                })
                .dedup()
                .collect::<Vec<_>>()
        } else {
            vec![]
        };

        let preset = FixturePreset::new(
            id,
            name,
            FixturePresetData::Default {
                data: discrete_data,
            },
            display_colors,
        )?;

        self.presets.insert(id, preset);
        event_list.push(DemexEvent::PoolItemAdded(
            PoolType::Preset(id.feature_group),
            id.preset_id,
        ));

        Ok(())
    }

    pub fn create_effect_preset(
        &mut self,
        id: FixturePresetId,
        name: Option<String>,
        event_list: &mut DemexEventList,
    ) -> Result<(), PresetHandlerError> {
        if self.presets.contains_key(&id) {
            return Err(PresetHandlerError::FeaturePresetAlreadyExists(id));
        }

        let preset = FixturePreset::new(
            id,
            name,
            FixturePresetData::FeatureEffect {
                runtime: FeatureEffectRuntime::new(Effect2::default()),
            },
            vec![],
        )?;

        self.presets.insert(id, preset);
        event_list.push(DemexEvent::PoolItemAdded(
            PoolType::Preset(id.feature_group),
            id.preset_id,
        ));

        Ok(())
    }

    pub fn update_preset(
        &mut self,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        id: FixturePresetId,
        patch: &Patch,
        fixture_handler: &FixtureStateHandler,
        timing_handler: &TimingHandler,
        update_mode: UpdateMode,
    ) -> Result<usize, PresetHandlerError> {
        let preset = self.get_preset(id)?;

        let discrete_data = FixturePreset::generate_preset_data(
            patch,
            fixture_handler,
            self,
            timing_handler,
            fixture_selector,
            fixture_selector_context,
            preset.id().feature_group,
        )?;

        let preset = self.get_preset_mut(id)?;

        let values_updated = preset.update(discrete_data, update_mode)?;

        Ok(values_updated)
    }

    pub fn move_preset(
        &mut self,
        id: FixturePresetId,
        target_preset: FixturePresetId,
        event_list: &mut DemexEventList,
    ) -> Result<(), PresetHandlerError> {
        let mut preset_to_move = self.get_preset(id)?.clone();

        let target_preset_result = self.get_preset(target_preset);

        if let Ok(target_preset) = target_preset_result {
            return Err(PresetHandlerError::PresetAlreadyExists(
                target_preset.id().preset_id,
            ));
        }

        preset_to_move.move_to(target_preset);
        self.presets.remove(&id);
        self.presets.insert(target_preset, preset_to_move);

        event_list.push(DemexEvent::PoolItemMoved {
            pool_type: PoolType::Preset(target_preset.feature_group),
            from_id: id.preset_id,
            to_id: target_preset.preset_id,
        });

        Ok(())
    }

    pub fn presets_mut(&mut self) -> &mut HashMap<FixturePresetId, FixturePreset> {
        &mut self.presets
    }

    pub fn presets(&self) -> &HashMap<FixturePresetId, FixturePreset> {
        &self.presets
    }

    pub fn presets_for_feature_group(
        &self,
        feature_group_id: FixtureChannel3FeatureGroup,
    ) -> Vec<&FixturePreset> {
        self.presets
            .values()
            .filter(|p| p.id().feature_group == feature_group_id)
            .collect()
    }

    pub fn next_preset_id(&self, feature_group_id: FixtureChannel3FeatureGroup) -> u32 {
        self.presets
            .keys()
            .filter(|k| k.feature_group == feature_group_id)
            .map(|k| k.preset_id)
            .max()
            .unwrap_or(0)
            + 1
    }

    pub fn get_preset_range(
        &self,
        preset_id_from: FixturePresetId,
        preset_id_to: FixturePresetId,
    ) -> Result<Vec<&FixturePreset>, PresetHandlerError> {
        let mut presets = Vec::new();

        if preset_id_from.feature_group != preset_id_to.feature_group {
            return Err(PresetHandlerError::FeatureGroupMismatch(
                preset_id_from.feature_group,
                preset_id_to.feature_group,
            ));
        }

        for i in preset_id_from.preset_id..=preset_id_to.preset_id {
            let preset = self.get_preset(FixturePresetId {
                feature_group: preset_id_from.feature_group,
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
        attribute: &FixtureChannel3Attribute,
        timing_handler: &TimingHandler,
        state: Option<&FixtureChannelValue2PresetState>,
    ) -> Option<FixtureChannelValue3> {
        let preset = self.get_preset(preset_id);

        if let Ok(preset) = preset {
            preset.value(fixture, attribute, self, timing_handler, state)
        } else {
            None
        }
    }

    pub fn apply_preset(
        &self,
        preset_id: FixturePresetId,
        fixture_handler: &mut FixtureStateHandler,
        patch: &Patch,
        selection: FixtureSelection,
    ) -> Result<(), PresetHandlerError> {
        let preset = self.get_preset(preset_id)?;

        for fixture_path in selection.fixtures() {
            let Ok(fixture) = patch.fixture(fixture_path) else {
                continue;
            };

            preset.apply(
                fixture,
                fixture_handler
                    .fixture_mut(fixture_path)
                    .map_err(PresetHandlerError::FixtureError)?,
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

    pub fn delete_preset_range(
        &mut self,
        preset_id_from: FixturePresetId,
        preset_id_to: FixturePresetId,
        event_list: &mut DemexEventList,
    ) -> Result<usize, PresetHandlerError> {
        if preset_id_from.feature_group != preset_id_to.feature_group {
            return Err(PresetHandlerError::FeatureGroupMismatch(
                preset_id_from.feature_group,
                preset_id_to.feature_group,
            ));
        }

        let mut count = 0;
        for id in preset_id_from.preset_id..=preset_id_to.preset_id {
            self.delete_preset(FixturePresetId {
                feature_group: preset_id_from.feature_group,
                preset_id: id,
            })?;
            count += 1;
        }

        event_list.push(DemexEvent::PoolItemsDeleted {
            pool_type: PoolType::Preset(preset_id_from.feature_group),
            from_id: preset_id_from.preset_id,
            to_id: preset_id_to.preset_id,
        });

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
        event_list: &mut DemexEventList,
    ) -> Result<(), PresetHandlerError> {
        if self.macros.contains_key(&id) {
            return Err(PresetHandlerError::PresetAlreadyExists(id));
        }

        self.macros.insert(id, MMacro::new(id, name, action));
        event_list.push(DemexEvent::PoolItemAdded(PoolType::Macro, id));

        Ok(())
    }

    pub fn get_macro(&self, id: u32) -> Result<&MMacro, PresetHandlerError> {
        self.macros
            .get(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))
    }

    pub fn get_macro_mut(&mut self, id: u32) -> Result<&mut MMacro, PresetHandlerError> {
        self.macros
            .get_mut(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))
    }

    pub fn next_macro_id(&self) -> u32 {
        self.macros.keys().max().unwrap_or(&0) + 1
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
        event_list: &mut DemexEventList,
    ) -> Result<(), PresetHandlerError> {
        if self.sequences.contains_key(&id) {
            return Err(PresetHandlerError::PresetAlreadyExists(id));
        }

        self.sequences.insert(
            id,
            Sequence::new(id, name.unwrap_or(format!("Sequence {}", id))),
        );
        event_list.push(DemexEvent::PoolItemAdded(PoolType::Sequence, id));

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
        fixture_handler: &FixtureStateHandler,
        channel_type_selector: &RecordChannelTypeSelector,
        update_mode: UpdateMode,
        patch: &Patch,
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

        let cue_data =
            Cue::generate_cue_data(patch, fixture_handler, &selection, channel_type_selector)?;

        let values_updated = cue.update(sequence_id, cue_data, &selection, update_mode)?;

        Ok(values_updated)
    }

    pub fn record_sequence_cue(
        &mut self,
        sequence_id: u32,
        fixture_handler: &FixtureStateHandler,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        cue_idx: Option<CueIdx>,
        channel_type_selector: &RecordChannelTypeSelector,
        fixture_types: &Patch,
        event_list: &mut DemexEventList,
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

        let cue_data = Cue::generate_cue_data(
            fixture_types,
            fixture_handler,
            &selection,
            channel_type_selector,
        )?;

        let cue = Cue::new(
            discrete_cue_idx,
            cue_data,
            selection,
            0.0,
            0.0,
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

                event_list.push(DemexEvent::PoolItemAdded(
                    PoolType::SequenceCue(sequence_id),
                    discrete_cue_idx.into(),
                ));
                return Ok(());
            }
        }

        // if we didn't insert the cue yet, it means it's the last cue
        cues.push(cue);
        event_list.push(DemexEvent::PoolItemAdded(
            PoolType::SequenceCue(sequence_id),
            discrete_cue_idx.into(),
        ));

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

    pub fn delete_sequence(
        &mut self,
        id: u32,
        event_list: &mut DemexEventList,
    ) -> Result<(), PresetHandlerError> {
        self.sequences
            .remove(&id)
            .ok_or(PresetHandlerError::PresetNotFound(id))?;
        event_list.push(DemexEvent::PoolItemsDeleted {
            pool_type: PoolType::Sequence,
            from_id: id,
            to_id: id,
        });

        Ok(())
    }

    pub fn delete_sequence_cues(
        &mut self,
        sequence_id: u32,
        cue_from: CueIdx,
        cue_to: CueIdx,
        event_list: &mut DemexEventList,
    ) -> Result<usize, PresetHandlerError> {
        if cue_from > cue_to {
            return Err(PresetHandlerError::InvalidCueRange(cue_from, cue_to));
        }

        let sequence = self
            .sequences
            .get_mut(&sequence_id)
            .ok_or(PresetHandlerError::PresetNotFound(sequence_id))?;

        let initial_len = sequence.cues().len();
        sequence
            .cues_mut()
            .retain(|c| c.cue_idx() < cue_from || c.cue_idx() > cue_to);

        event_list.push(DemexEvent::PoolItemsDeleted {
            pool_type: PoolType::SequenceCue(sequence_id),
            from_id: cue_from.into(),
            to_id: cue_to.into(),
        });

        Ok(initial_len - sequence.cues().len())
    }
}

impl Pool for PresetHandler {
    fn get(
        &self,
        pool_type: crate::pool::PoolType,
        id: u32,
    ) -> Result<crate::pool::PoolItem, crate::pool::PoolError> {
        match pool_type {
            PoolType::Preset(preset_type) => {
                let preset_id = FixturePresetId::new(preset_type, id);
                let item = self
                    .presets
                    .get(&preset_id)
                    .ok_or(PoolError::PoolItemNotFound(pool_type, id))?;
                Ok(item.into())
            }
            PoolType::Sequence => Ok(self
                .sequences
                .get(&id)
                .ok_or(PoolError::PoolItemNotFound(pool_type, id))?
                .into()),
            PoolType::Group => Ok(self
                .groups
                .get(&id)
                .ok_or(PoolError::PoolItemNotFound(pool_type, id))?
                .into()),
            PoolType::Macro => Ok(self
                .macros
                .get(&id)
                .ok_or(PoolError::PoolItemNotFound(pool_type, id))?
                .into()),
            pool_type => Err(PoolError::InvalidPoolType(pool_type)),
        }
    }

    fn get_all(&self, pool_type: PoolType) -> Result<Vec<crate::pool::PoolItem>, PoolError> {
        match pool_type {
            PoolType::Preset(preset_type) => {
                let presets = self
                    .presets
                    .values()
                    .filter(|p| p.id().feature_group == preset_type)
                    .map(|p| p.into())
                    .collect();

                Ok(presets)
            }
            PoolType::Sequence => Ok(self.sequences.values().map(|s| s.into()).collect()),
            PoolType::Group => Ok(self.groups.values().map(|s| s.into()).collect()),
            PoolType::Macro => Ok(self.macros.values().map(|s| s.into()).collect()),
            pool_type => Err(PoolError::InvalidPoolType(pool_type)),
        }
    }

    fn set_name(
        &mut self,
        _pool_type: crate::pool::PoolType,
        _id: u32,
        _name: String,
    ) -> Result<(), crate::pool::PoolError> {
        todo!()
    }
}
