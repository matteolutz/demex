use serde::{Deserialize, Serialize};

use crate::{
    fixture::{
        channel2::feature::{feature_type::FixtureFeatureType, feature_value::FixtureFeatureValue},
        error::FixtureError,
        handler::FixtureHandler,
        presets::PresetHandler,
        sequence::cue::{CueFixtureChannelValue, CueIdx},
        updatables::UpdatableHandler,
        Fixture,
    },
    ui::{constants::INFO_TEXT, window::edit::DemexEditWindow},
};

use self::{error::ActionRunError, result::ActionRunResult};

use super::{
    fixture_selector::{FixtureSelector, FixtureSelectorContext, FixtureSelectorError},
    object::HomeableObject,
};

pub mod error;
pub mod result;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum CueIdxSelectorActionData {
    Discrete(CueIdx),
    Next,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelTypeSelectorActionData {
    All,
    Active,
    Features(Vec<FixtureFeatureType>),
}

impl ChannelTypeSelectorActionData {
    pub fn get_channel_values(
        &self,
        fixture: &Fixture,
    ) -> Result<Vec<CueFixtureChannelValue>, FixtureError> {
        let mut values = Vec::new();

        for channel_type in fixture.channel_types() {
            match self {
                Self::All => {
                    values.push(CueFixtureChannelValue::new(
                        fixture.channel_value_programmer(*channel_type)?.clone(),
                        *channel_type,
                        false,
                    ));
                }
                Self::Active => {
                    let value = fixture.channel_value_programmer(*channel_type)?;
                    if value.is_home() {
                        continue;
                    }

                    values.push(CueFixtureChannelValue::new(
                        value.clone(),
                        *channel_type,
                        false,
                    ));
                }
                Self::Features(_channels) => {
                    todo!()
                }
            }
        }

        Ok(values)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelValueSingleActionData {
    Single(f32),
    Thru(f32, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UpdateModeActionData {
    Merge,
    Override,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConfigTypeActionData {
    Output,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FaderCreationConfigActionData {
    Submaster(FixtureSelector),
    Sequence(u32, FixtureSelector),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum Action {
    SetFeatureValue(
        FixtureSelector,
        FixtureFeatureType,
        ChannelValueSingleActionData,
    ),
    SetChannelValuePreset(FixtureSelector, u32),
    SetChannelValuePresetRange(FixtureSelector, u32, u32),

    Home(HomeableObject),
    HomeAll,

    RecordPreset(u32, Option<u32>, FixtureSelector, Option<String>),
    RecordGroup2(Option<u32>, FixtureSelector, Option<String>),
    RecordSequenceCue(
        u32,
        CueIdxSelectorActionData,
        FixtureSelector,
        ChannelTypeSelectorActionData,
    ),

    RenamePreset(u32, String),
    RenameGroup(u32, String),
    RenameSequence(u32, String),

    RecordMacro(Box<Action>, u32),

    CreateSequence(Option<u32>, Option<String>),
    CreateExecutor(Option<u32>, u32, FixtureSelector),
    CreateMacro(Option<u32>, Box<Action>, Option<String>),
    CreateFader(Option<u32>, FaderCreationConfigActionData, Option<String>),

    UpdatePreset(u32, FixtureSelector, UpdateModeActionData),

    DeleteMacro((u32, u32)),
    DeletePreset((u32, u32)),
    DeleteSequence((u32, u32)),
    DeleteExecutor((u32, u32)),
    DeleteFader((u32, u32)),
    DeleteGroup((u32, u32)),

    EditSequence(u32),
    EditSequenceCue(u32, CueIdx),
    EditExecutor(u32),
    EditFader(u32),
    EditPreset(u32),

    FixtureSelector(FixtureSelector),
    ClearAll,
    Save,
    Test(String),

    Config(ConfigTypeActionData),

    Nuzul,
    Sueud,

    #[default]
    MatteoLutz,
}

impl Action {
    pub fn run(
        &self,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &mut PresetHandler,
        fixture_selector_context: FixtureSelectorContext,
        updatable_handler: &mut UpdatableHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        match self {
            Self::SetFeatureValue(fixture_selector, channel_type, value) => self
                .run_set_channel_value(
                    preset_handler,
                    fixture_handler,
                    fixture_selector,
                    fixture_selector_context,
                    *channel_type,
                    value,
                ),
            Self::SetChannelValuePreset(fixture_selector, preset_id) => self
                .run_set_channel_value_preset(
                    preset_handler,
                    fixture_handler,
                    fixture_selector,
                    fixture_selector_context,
                    *preset_id,
                ),
            Self::SetChannelValuePresetRange(fixture_selector, preset_id_from, preset_id_to) => {
                self.run_set_channel_value_preset_range(
                    preset_handler,
                    fixture_handler,
                    fixture_selector,
                    fixture_selector_context,
                    *preset_id_from,
                    *preset_id_to,
                )
            }

            Self::Home(homeable_object) => self.run_home(
                preset_handler,
                fixture_handler,
                updatable_handler,
                homeable_object,
                fixture_selector_context,
            ),
            Self::HomeAll => self.run_home_all(fixture_handler),
            Self::RecordPreset(feature_group_id, id, fixture_selector, name) => self
                .run_record_preset(
                    preset_handler,
                    fixture_handler,
                    fixture_selector,
                    fixture_selector_context,
                    *feature_group_id,
                    *id,
                    name,
                ),
            Self::RecordGroup2(id, fixture_selector, name) => self.run_record_group(
                preset_handler,
                fixture_selector,
                fixture_selector_context,
                *id,
                name,
            ),
            Self::RecordSequenceCue(
                sequence_id,
                cue_idx,
                fixture_selector,
                channel_type_selector,
            ) => self.run_record_sequence_cue(
                preset_handler,
                fixture_handler,
                fixture_selector,
                fixture_selector_context,
                *sequence_id,
                *cue_idx,
                channel_type_selector,
            ),

            Self::RenamePreset(id, new_name) => {
                self.run_rename_preset(preset_handler, *id, new_name)
            }
            Self::RenameGroup(id, new_name) => self.run_rename_group(preset_handler, *id, new_name),
            Self::RenameSequence(id, new_name) => {
                self.run_rename_sequence(preset_handler, *id, new_name)
            }

            Self::CreateSequence(id, name) => self.run_create_sequence(preset_handler, *id, name),
            Self::CreateExecutor(id, sequence_id, fixture_selector) => self.run_create_executor(
                updatable_handler,
                preset_handler,
                *id,
                *sequence_id,
                fixture_selector,
                fixture_selector_context,
            ),
            Self::CreateMacro(id, action, name) => {
                self.run_create_macro(*id, action, name, preset_handler)
            }
            Self::CreateFader(id, config, name) => self.run_create_fader(
                *id,
                config,
                name,
                updatable_handler,
                preset_handler,
                fixture_selector_context,
            ),

            Self::UpdatePreset(preset_id, fixture_selector, update_mode) => self.run_update_preset(
                preset_handler,
                fixture_handler,
                fixture_selector,
                fixture_selector_context,
                *preset_id,
                update_mode,
            ),

            Self::DeleteMacro(id_range) => self.run_delete_macro(*id_range, preset_handler),
            Self::DeletePreset(id_range) => self.run_delete_preset(*id_range, preset_handler),
            Self::DeleteSequence(sequence_range) => {
                self.run_delete_sequence(*sequence_range, preset_handler, updatable_handler)
            }
            Self::DeleteExecutor(executor_range) => {
                self.run_delete_executor(*executor_range, updatable_handler)
            }
            Self::DeleteFader(fader_range) => {
                self.run_delete_fader(*fader_range, updatable_handler)
            }
            Self::DeleteGroup(group_range) => self.run_delete_group(*group_range, preset_handler),

            Self::EditSequence(sequence_id) => Ok(ActionRunResult::EditWindow(
                DemexEditWindow::EditSequence(*sequence_id),
            )),
            Self::EditSequenceCue(sequence_id, cue_idx) => Ok(ActionRunResult::EditWindow(
                DemexEditWindow::EditSequenceCue(*sequence_id, *cue_idx),
            )),
            Self::EditExecutor(executor_id) => Ok(ActionRunResult::EditWindow(
                DemexEditWindow::EditExecutor(*executor_id),
            )),
            Self::EditFader(fader_id) => Ok(ActionRunResult::EditWindow(
                DemexEditWindow::EditFader(*fader_id),
            )),
            Self::EditPreset(preset_id) => Ok(ActionRunResult::EditWindow(
                DemexEditWindow::EditPreset(*preset_id),
            )),

            Self::ClearAll => Ok(ActionRunResult::new()),
            Self::FixtureSelector(fixture_selector) => self.run_fixture_selector(
                fixture_selector,
                fixture_selector_context,
                preset_handler,
                fixture_handler,
            ),
            Self::Test(_) => Ok(ActionRunResult::new()),
            Self::Save => Ok(ActionRunResult::new()),

            Self::Nuzul => Ok(ActionRunResult::Info("Going down...".to_owned())),
            Self::Sueud => Ok(ActionRunResult::Info("Going up...".to_owned())),

            Self::Config(config_type) => Ok(ActionRunResult::EditWindow(DemexEditWindow::Config(
                *config_type,
            ))),

            Self::MatteoLutz => Ok(ActionRunResult::InfoWithLink(
                INFO_TEXT.to_owned(),
                "https://matteolutz.de".to_owned(),
            )),

            unimplemented_action => Err(ActionRunError::UnimplementedAction(
                unimplemented_action.clone(),
            )),
        }
    }

    fn run_set_channel_value(
        &self,
        preset_handler: &PresetHandler,
        fixture_handler: &mut FixtureHandler,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        feature_type: FixtureFeatureType,
        value: &ChannelValueSingleActionData,
    ) -> Result<ActionRunResult, ActionRunError> {
        let fixtures = fixture_selector
            .get_fixtures(preset_handler, fixture_selector_context)
            .map_err(ActionRunError::FixtureSelectorError)?;

        for (idx, fixture_id) in fixtures.iter().enumerate() {
            let discrete_value = match value {
                ChannelValueSingleActionData::Single(value) => *value,
                ChannelValueSingleActionData::Thru(start, end) => {
                    let range = end - start;
                    let step = range / (fixtures.len() - 1) as f32;
                    start + step * idx as f32
                }
            };

            if let Some(fixture) = fixture_handler.fixture(*fixture_id) {
                match feature_type {
                    FixtureFeatureType::Intensity => fixture
                        .set_feature_value(FixtureFeatureValue::Intensity {
                            intensity: discrete_value,
                        })
                        .map_err(ActionRunError::FixtureError)?,
                    unhandled_feature_type => {
                        Err(ActionRunError::Todo(format!(
                            "Handle set of feature type {:?}",
                            unhandled_feature_type
                        )))?;
                    }
                }
            }
        }

        Ok(ActionRunResult::new())
    }

    fn run_set_channel_value_preset(
        &self,
        preset_handler: &PresetHandler,
        fixture_handler: &mut FixtureHandler,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        preset_id: u32,
    ) -> Result<ActionRunResult, ActionRunError> {
        let fixtures = fixture_selector
            .get_fixtures(preset_handler, fixture_selector_context)
            .map_err(ActionRunError::FixtureSelectorError)?;

        let preset = preset_handler
            .get_preset(preset_id)
            .map_err(ActionRunError::PresetHandlerError)?;

        for fixture in fixtures {
            if let Some(f) = fixture_handler.fixture(fixture) {
                preset
                    .apply(f)
                    .map_err(ActionRunError::PresetHandlerError)?;
            }
        }

        Ok(ActionRunResult::new())
    }

    fn run_set_channel_value_preset_range(
        &self,
        _preset_handler: &PresetHandler,
        _fixture_handler: &mut FixtureHandler,
        _fixture_selector: &FixtureSelector,
        _fixture_selector_context: FixtureSelectorContext,
        _preset_id_from: u32,
        _preset_id_to: u32,
    ) -> Result<ActionRunResult, ActionRunError> {
        /*
        let fixtures = fixture_selector
            .get_fixtures(preset_handler, fixture_selector_context)
            .map_err(ActionRunError::FixtureSelectorError)?;

        let presets = preset_handler
            .get_preset_range(preset_id_from, preset_id_to, channel_type)
            .map_err(ActionRunError::PresetHandlerError)?;

        for (idx, fixture) in fixtures.iter().enumerate() {
            if let Some(f) = fixture_handler.fixture(*fixture) {
                // get the two relevant indexes from the presets
                let preset_idx_fl =
                    idx as f32 * ((presets.len() - 1) as f32 / fixtures.len() as f32);

                let preset_idx_low = preset_idx_fl.floor() as usize;
                let preset_idx_high = preset_idx_low + 1;

                let fade = (idx as f32 * ((presets.len()) as f32 / fixtures.len() as f32))
                    - preset_idx_low as f32;

                let channel_value = FixtureChannelValue::Mix {
                    a: Box::new(FixtureChannelValue::Preset(presets[preset_idx_low].id())),
                    b: Box::new(FixtureChannelValue::Preset(presets[preset_idx_high].id())),
                    mix: fade,
                };

                f.set_channel_value(channel_type, channel_value)
                    .map_err(ActionRunError::FixtureError)?;
            }
        }
        */

        Ok(ActionRunResult::new())
    }

    fn run_home_all(
        &self,
        fixture_handler: &mut FixtureHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        fixture_handler
            .home_all()
            .map_err(ActionRunError::FixtureHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_home(
        &self,
        preset_handler: &PresetHandler,
        fixture_handler: &mut FixtureHandler,
        updatable_handler: &mut UpdatableHandler,
        homeable_object: &HomeableObject,
        fixture_selector_context: FixtureSelectorContext,
    ) -> Result<ActionRunResult, ActionRunError> {
        match homeable_object {
            HomeableObject::FixtureSelector(fixture_selector) => {
                let fixtures = fixture_selector
                    .get_fixtures(preset_handler, fixture_selector_context)
                    .map_err(ActionRunError::FixtureSelectorError)?;

                for fixture_id in fixtures {
                    if let Some(fixture) = fixture_handler.fixture(fixture_id) {
                        fixture.home().map_err(ActionRunError::FixtureError)?;
                    }
                }

                Ok(ActionRunResult::new())
            }
            HomeableObject::Executor(executor_id) => {
                updatable_handler
                    .stop_executor(*executor_id, fixture_handler)
                    .map_err(ActionRunError::UpdatableHandlerError)?;

                Ok(ActionRunResult::new())
            }
            HomeableObject::Fader(fader_id) => {
                if let Ok(fader) = updatable_handler.fader_mut(*fader_id) {
                    fader.home(fixture_handler);
                }

                Ok(ActionRunResult::new())
            }
        }
    }

    fn run_record_preset(
        &self,
        preset_handler: &mut PresetHandler,
        fixture_handler: &FixtureHandler,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        feature_group_id: u32,
        id: Option<u32>,
        name: &Option<String>,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .record_preset(
                fixture_selector,
                fixture_selector_context,
                id.unwrap_or_else(|| preset_handler.next_preset_id()),
                name.clone(),
                fixture_handler,
                feature_group_id,
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_record_group(
        &self,
        preset_handler: &mut PresetHandler,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        id: Option<u32>,
        name: &Option<String>,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .record_group(
                fixture_selector
                    .flatten(preset_handler, fixture_selector_context)
                    .map_err(ActionRunError::FixtureSelectorError)?,
                id.unwrap_or_else(|| preset_handler.next_group_id()),
                name.clone(),
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_record_sequence_cue(
        &self,
        preset_handler: &mut PresetHandler,
        fixture_handler: &FixtureHandler,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        sequence_id: u32,
        cue_idx: CueIdxSelectorActionData,
        channel_type_selector: &ChannelTypeSelectorActionData,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .record_sequence_cue(
                sequence_id,
                fixture_handler,
                fixture_selector,
                fixture_selector_context,
                cue_idx,
                channel_type_selector,
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_rename_group(
        &self,
        preset_handler: &mut PresetHandler,
        id: u32,
        new_name: &str,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .rename_group(id, new_name.to_owned())
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_rename_sequence(
        &self,
        preset_handler: &mut PresetHandler,
        id: u32,
        new_name: &str,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .rename_sequence(id, new_name.to_owned())
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_rename_preset(
        &self,
        preset_handler: &mut PresetHandler,
        id: u32,
        new_name: &str,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .rename_preset(id, new_name.to_owned())
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_create_sequence(
        &self,
        preset_handler: &mut PresetHandler,
        id: Option<u32>,
        name: &Option<String>,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .create_sequence(
                id.unwrap_or_else(|| preset_handler.next_sequence_id()),
                name.clone(),
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_create_executor(
        &self,
        updatable_handler: &mut UpdatableHandler,
        preset_handler: &PresetHandler,
        id: Option<u32>,
        sequence_id: u32,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
    ) -> Result<ActionRunResult, ActionRunError> {
        let fixtures_selected = fixture_selector
            .get_fixtures(preset_handler, fixture_selector_context)
            .map_err(ActionRunError::FixtureSelectorError)?;

        updatable_handler
            .create_executor(
                id.unwrap_or_else(|| updatable_handler.next_executor_id()),
                sequence_id,
                fixtures_selected,
            )
            .map_err(ActionRunError::UpdatableHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_create_macro(
        &self,
        id: Option<u32>,
        action: &Action,
        name: &Option<String>,
        preset_handler: &mut PresetHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .create_macro(
                id.unwrap_or_else(|| preset_handler.next_macro_id()),
                name.clone(),
                Box::new(action.clone()),
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_create_fader(
        &self,
        id: Option<u32>,
        config: &FaderCreationConfigActionData,
        name: &Option<String>,
        updatable_handler: &mut UpdatableHandler,
        preset_handler: &PresetHandler,
        fixture_selector_context: FixtureSelectorContext,
    ) -> Result<ActionRunResult, ActionRunError> {
        updatable_handler
            .create_fader(
                id.unwrap_or_else(|| updatable_handler.next_fader_id()),
                config,
                name.clone(),
                preset_handler,
                fixture_selector_context,
            )
            .map_err(ActionRunError::UpdatableHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_update_preset(
        &self,
        preset_handler: &mut PresetHandler,
        fixture_handler: &mut FixtureHandler,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        preset_id: u32,
        update_mode: &UpdateModeActionData,
    ) -> Result<ActionRunResult, ActionRunError> {
        let num_updated = preset_handler
            .update_preset(
                fixture_selector,
                fixture_selector_context,
                preset_id,
                fixture_handler,
                update_mode,
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        if num_updated == 0 {
            Ok(ActionRunResult::Warn(
                "No fixtures we're affected. If you're trying override existing preset data, try running with the \"override\" flag.".to_owned(),
            ))
        } else {
            Ok(ActionRunResult::new())
        }
    }

    fn run_fixture_selector(
        &self,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        preset_handler: &PresetHandler,
        fixture_handler: &FixtureHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        // flatten the fixture selector, so we don't have
        // outdated references to the previously selected fixtures
        let fixture_selector = fixture_selector
            .flatten(preset_handler, fixture_selector_context.clone())
            .map_err(ActionRunError::FixtureSelectorError)?;

        let selected_fixtures = fixture_selector
            .get_fixtures(preset_handler, fixture_selector_context)
            .map_err(ActionRunError::FixtureSelectorError)?;

        if selected_fixtures.is_empty() {
            return Err(ActionRunError::FixtureSelectorError(
                FixtureSelectorError::NoFixturesMatched,
            ));
        }

        let unknown_fixtures = selected_fixtures
            .into_iter()
            .filter(|f_id| !fixture_handler.has_fixture(*f_id))
            .collect::<Vec<_>>();
        if !unknown_fixtures.is_empty() {
            return Err(ActionRunError::FixtureSelectorError(
                FixtureSelectorError::SomeFixturesFailedToMatch(unknown_fixtures),
            ));
        }

        Ok(ActionRunResult::UpdateSelectedFixtures(fixture_selector))
    }

    pub fn run_delete_macro(
        &self,
        (id_from, id_to): (u32, u32),
        preset_handler: &mut PresetHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        for macro_id in id_from..=id_to {
            preset_handler
                .delete_macro(macro_id)
                .map_err(ActionRunError::PresetHandlerError)?;
        }

        if id_from == id_to {
            Ok(ActionRunResult::new())
        } else {
            Ok(ActionRunResult::Info(format!(
                "Deleted {} macros",
                id_to - id_from + 1
            )))
        }
    }

    pub fn run_delete_preset(
        &self,
        (id_from, id_to): (u32, u32),
        preset_handler: &mut PresetHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        // TODO: what happens with data referring to this preset?
        for id in id_from..=id_to {
            preset_handler
                .delete_preset(id)
                .map_err(ActionRunError::PresetHandlerError)?;
        }

        if id_from == id_to {
            Ok(ActionRunResult::new())
        } else {
            Ok(ActionRunResult::Info(format!(
                "Deleted {} presets",
                id_to - id_from + 1
            )))
        }
    }

    pub fn run_delete_sequence(
        &self,
        (id_from, id_to): (u32, u32),
        preset_handler: &mut PresetHandler,
        updatable_handler: &mut UpdatableHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        for id in id_from..=id_to {
            if !updatable_handler.sequence_deleteable(id) {
                return Err(ActionRunError::SequenceDeleteDependencies(id));
            }
        }

        for id in id_from..=id_to {
            preset_handler
                .delete_sequence(id)
                .map_err(ActionRunError::PresetHandlerError)?;
        }

        if id_from == id_to {
            Ok(ActionRunResult::new())
        } else {
            Ok(ActionRunResult::Info(format!(
                "Deleted {} sequences",
                id_to - id_from + 1
            )))
        }
    }

    pub fn run_delete_executor(
        &self,
        (id_from, id_to): (u32, u32),
        updatable_handler: &mut UpdatableHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        for id in id_from..=id_to {
            if updatable_handler
                .executor(id)
                .is_some_and(|exec| exec.is_started())
            {
                return Err(ActionRunError::ExecutorIsRunning(id));
            }
        }

        for id in id_from..=id_to {
            updatable_handler
                .delete_executor(id)
                .map_err(ActionRunError::UpdatableHandlerError)?;
        }

        if id_from == id_to {
            Ok(ActionRunResult::new())
        } else {
            Ok(ActionRunResult::Info(format!(
                "Deleted {} executors",
                id_to - id_from + 1
            )))
        }
    }

    pub fn run_delete_fader(
        &self,
        (id_from, id_to): (u32, u32),
        updatable_handler: &mut UpdatableHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        for id in id_from..=id_to {
            if updatable_handler
                .fader(id)
                .is_ok_and(|fader| fader.is_active())
            {
                return Err(ActionRunError::FaderIsActive(id));
            }
        }

        for id in id_from..=id_to {
            updatable_handler
                .delete_fader(id)
                .map_err(ActionRunError::UpdatableHandlerError)?;
        }

        if id_from == id_to {
            Ok(ActionRunResult::new())
        } else {
            Ok(ActionRunResult::Info(format!(
                "Deleted {} faders",
                id_to - id_from + 1
            )))
        }
    }

    pub fn run_delete_group(
        &self,
        (id_from, id_to): (u32, u32),
        preset_handler: &mut PresetHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        for id in id_from..=id_to {
            preset_handler
                .delete_group(id)
                .map_err(ActionRunError::PresetHandlerError)?;
        }

        if id_from == id_to {
            Ok(ActionRunResult::new())
        } else {
            Ok(ActionRunResult::Info(format!(
                "Deleted {} groups",
                id_to - id_from + 1
            )))
        }
    }
}
