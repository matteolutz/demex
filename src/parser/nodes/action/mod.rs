use serde::{Deserialize, Serialize};

use crate::{
    fixture::{
        channel::value::{
            FixtureChannelDiscreteValue, FixtureChannelValue, FixtureChannelValueTrait,
        },
        error::FixtureError,
        handler::FixtureHandler,
        presets::PresetHandler,
        sequence::cue::{CueFixtureChannelValue, CueIdx},
        updatables::UpdatableHandler,
        Fixture,
    },
    ui::{constants::INFO_TEXT, edit::DemexEditWindow},
};

use self::{error::ActionRunError, result::ActionRunResult};

use super::{
    fixture_selector::{FixtureSelector, FixtureSelectorContext},
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
    Channels(Vec<u16>),
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
                        fixture.channel_value_programmer(*channel_type)?,
                        *channel_type,
                        false,
                    ));
                }
                Self::Active => {
                    let value = fixture.channel_value_programmer(*channel_type)?;
                    if value.is_home() {
                        continue;
                    }

                    values.push(CueFixtureChannelValue::new(value, *channel_type, false));
                }
                Self::Channels(channels) => {
                    if channels.contains(channel_type) {
                        values.push(CueFixtureChannelValue::new(
                            fixture.channel_value_programmer(*channel_type)?,
                            *channel_type,
                            false,
                        ));
                    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    SetChannelValue(FixtureSelector, u16, ChannelValueSingleActionData),
    SetChannelValuePreset(FixtureSelector, u16, u32),
    SetChannelValuePresetRange(FixtureSelector, u16, u32, u32),

    Home(HomeableObject),
    HomeAll,

    RecordPreset(u16, u32, FixtureSelector, Option<String>),
    RecordGroup2(u32, FixtureSelector, Option<String>),
    RecordSequenceCue(
        u32,
        CueIdxSelectorActionData,
        FixtureSelector,
        ChannelTypeSelectorActionData,
    ),

    RenamePreset(u16, u32, String),
    RenameGroup(u32, String),
    RenameSequence(u32, String),

    RecordMacro(Box<Action>, u32),

    CreateSequence(u32, Option<String>),
    CreateExecutor(u32, u32),
    CreateMacro(u32, Box<Action>, Option<String>),

    UpdatePreset(u16, u32, FixtureSelector, UpdateModeActionData),

    DeleteMacro((u32, u32)),

    EditSequence(u32),
    EditSequenceCue(u32, CueIdx),
    EditExecutor(u32),
    EditFader(u32),
    EditPreset(u16, u32),

    FixtureSelector(FixtureSelector),
    ClearAll,
    Save,
    Test(String),

    Config(ConfigTypeActionData),

    Nuzul,
    Sueud,

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
            Self::SetChannelValue(fixture_selector, channel_type, value) => self
                .run_set_channel_value(
                    preset_handler,
                    fixture_handler,
                    fixture_selector,
                    fixture_selector_context,
                    *channel_type,
                    value,
                ),
            Self::SetChannelValuePreset(fixture_selector, channel_type, preset_id) => self
                .run_set_channel_value_preset(
                    preset_handler,
                    fixture_handler,
                    fixture_selector,
                    fixture_selector_context,
                    *channel_type,
                    *preset_id,
                ),
            Self::SetChannelValuePresetRange(
                fixture_selector,
                channel_type,
                preset_id_from,
                preset_id_to,
            ) => self.run_set_channel_value_preset_range(
                preset_handler,
                fixture_handler,
                fixture_selector,
                fixture_selector_context,
                *channel_type,
                *preset_id_from,
                *preset_id_to,
            ),

            Self::Home(homeable_object) => self.run_home(
                preset_handler,
                fixture_handler,
                updatable_handler,
                homeable_object,
                fixture_selector_context,
            ),
            Self::HomeAll => self.run_home_all(fixture_handler),
            Self::RecordPreset(channel_type, id, fixture_selector, name) => self.run_record_preset(
                preset_handler,
                fixture_handler,
                updatable_handler,
                fixture_selector,
                fixture_selector_context,
                *channel_type,
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

            Self::RenamePreset(channel_type, id, new_name) => {
                self.run_rename_preset(preset_handler, *channel_type, *id, new_name)
            }
            Self::RenameGroup(id, new_name) => self.run_rename_group(preset_handler, *id, new_name),
            Self::RenameSequence(id, new_name) => {
                self.run_rename_sequence(preset_handler, *id, new_name)
            }

            Self::CreateSequence(id, name) => self.run_create_sequence(preset_handler, *id, name),
            Self::CreateExecutor(id, sequence_id) => {
                self.run_create_executor(updatable_handler, *id, *sequence_id)
            }
            Self::CreateMacro(id, action, name) => {
                self.run_create_macro(*id, action, name, preset_handler)
            }

            Self::UpdatePreset(channel_type, preset_id, fixture_selector, update_mode) => self
                .run_update_preset(
                    preset_handler,
                    updatable_handler,
                    fixture_handler,
                    fixture_selector,
                    fixture_selector_context,
                    *channel_type,
                    *preset_id,
                    update_mode,
                ),

            Self::DeleteMacro((id_from, id_to)) => {
                for macro_id in *id_from..=*id_to {
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
            Self::EditPreset(channel_type, preset_id) => Ok(ActionRunResult::EditWindow(
                DemexEditWindow::EditPreset(*channel_type, *preset_id),
            )),

            Self::ClearAll => Ok(ActionRunResult::new()),
            Self::FixtureSelector(_) => Ok(ActionRunResult::new()),
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
        channel_type: u16,
        value: &ChannelValueSingleActionData,
    ) -> Result<ActionRunResult, ActionRunError> {
        let fixtures = fixture_selector
            .get_fixtures(preset_handler, fixture_selector_context)
            .map_err(ActionRunError::FixtureSelectorError)?;

        for (idx, fixture) in fixtures.iter().enumerate() {
            let discrete_value = match value {
                ChannelValueSingleActionData::Single(value) => *value,
                ChannelValueSingleActionData::Thru(start, end) => {
                    let range = end - start;
                    let step = range / (fixtures.len() - 1) as f32;
                    start + step * idx as f32
                }
            };

            if let Some(f) = fixture_handler.fixture(*fixture) {
                f.set_channel_value(
                    channel_type,
                    FixtureChannelValue::Discrete(FixtureChannelDiscreteValue::Single(
                        discrete_value,
                    )),
                )
                .map_err(ActionRunError::FixtureError)?;
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
        channel_type: u16,
        preset_id: u32,
    ) -> Result<ActionRunResult, ActionRunError> {
        let fixtures = fixture_selector
            .get_fixtures(preset_handler, fixture_selector_context)
            .map_err(ActionRunError::FixtureSelectorError)?;

        let preset = preset_handler
            .get_preset(preset_id, channel_type)
            .map_err(ActionRunError::PresetHandlerError)?;

        for fixture in fixtures {
            if let Some(f) = fixture_handler.fixture(fixture) {
                f.set_channel_value(channel_type, FixtureChannelValue::Preset(preset.id()))
                    .map_err(ActionRunError::FixtureError)?;
            }
        }

        Ok(ActionRunResult::new())
    }

    fn run_set_channel_value_preset_range(
        &self,
        preset_handler: &PresetHandler,
        fixture_handler: &mut FixtureHandler,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        channel_type: u16,
        preset_id_from: u32,
        preset_id_to: u32,
    ) -> Result<ActionRunResult, ActionRunError> {
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
                if let Some(executor) = updatable_handler.executor_mut(*executor_id) {
                    executor.stop(fixture_handler, preset_handler);
                }

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
        updatable_handler: &UpdatableHandler,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        channel_type: u16,
        id: u32,
        name: &Option<String>,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .record_preset(
                fixture_selector,
                fixture_selector_context,
                id,
                name.clone(),
                fixture_handler,
                channel_type,
                updatable_handler,
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_record_group(
        &self,
        preset_handler: &mut PresetHandler,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        id: u32,
        name: &Option<String>,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .record_group(
                fixture_selector
                    .flatten(preset_handler, fixture_selector_context)
                    .map_err(ActionRunError::FixtureSelectorError)?,
                id,
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
        channel_type: u16,
        id: u32,
        new_name: &str,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .rename_preset(id, channel_type, new_name.to_owned())
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_create_sequence(
        &self,
        preset_handler: &mut PresetHandler,
        id: u32,
        name: &Option<String>,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .create_sequence(id, name.clone())
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_create_executor(
        &self,
        updatable_handler: &mut UpdatableHandler,
        id: u32,
        sequence_id: u32,
    ) -> Result<ActionRunResult, ActionRunError> {
        updatable_handler
            .create_executor(id, sequence_id)
            .map_err(ActionRunError::UpdatableHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_create_macro(
        &self,
        id: u32,
        action: &Action,
        name: &Option<String>,
        preset_handler: &mut PresetHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .create_macro(id, name.clone(), Box::new(action.clone()))
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_update_preset(
        &self,
        preset_handler: &mut PresetHandler,
        updatable_handler: &UpdatableHandler,
        fixture_handler: &mut FixtureHandler,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        channel_type: u16,
        preset_id: u32,
        update_mode: &UpdateModeActionData,
    ) -> Result<ActionRunResult, ActionRunError> {
        let num_updated = preset_handler
            .update_preset(
                fixture_selector,
                fixture_selector_context,
                channel_type,
                preset_id,
                fixture_handler,
                updatable_handler,
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
}
