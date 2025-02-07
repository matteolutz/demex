use serde::{Deserialize, Serialize};

use crate::fixture::{
    channel::value::{FixtureChannelDiscreteValue, FixtureChannelValue},
    handler::FixtureHandler,
    presets::PresetHandler,
    updatables::UpdatableHandler,
};

use self::{error::ActionRunError, result::ActionRunResult};

use super::{
    fixture_selector::{FixtureSelector, FixtureSelectorContext},
    object::HomeableObject,
};

pub mod error;
pub mod result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelTypeSelector {
    All,
    Active,
    Channels(Vec<u16>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelValueSingleActionData {
    Single(f32),
    Thru(f32, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    SetIntensity(FixtureSelector, f32),      // depr
    SetColor(FixtureSelector, [f32; 4]),     // depr
    SetColorPreset(FixtureSelector, u32),    // depr
    SetPosition(FixtureSelector, [f32; 2]),  // depr
    SetPositionPreset(FixtureSelector, u32), // depr

    SetChannelValue(FixtureSelector, u16, ChannelValueSingleActionData),
    SetChannelValuePreset(FixtureSelector, u16, u32),

    Home(HomeableObject),
    HomeAll,

    GoHome(FixtureSelector), // depr
    GoHomeAll,               // depr

    ManSet(FixtureSelector, String, f32), // depr

    RecordPreset(u16, u32, FixtureSelector, Option<String>),
    RecordGroup2(u32, FixtureSelector, Option<String>),
    RecordSequenceCue(u32, usize, FixtureSelector, ChannelTypeSelector),

    RecordGroup(FixtureSelector, u32), // depr

    RecordColor(FixtureSelector, u32),    // depr
    RecordPosition(FixtureSelector, u32), // depr

    RenamePreset(u16, u32, String),
    RenameGroup(u32, String),
    RenameSequence(u32, String),

    RecordMacro(Box<Action>, u32),

    RenameColorPreset(u32, String),    // depr
    RenamePositionPreset(u32, String), // depr

    CreateSequence(u32, Option<String>),
    CreateExecutor(u32, u32),

    FixtureSelector(FixtureSelector),
    ClearAll,
    Test(String),
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

            Self::Home(homeable_object) => self.run_home(
                preset_handler,
                fixture_handler,
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

            Self::RecordMacro(action, id) => self.run_record_macro(*id, action, preset_handler),
            Self::ClearAll => Ok(ActionRunResult::new()),
            Self::FixtureSelector(_) => Ok(ActionRunResult::new()),
            Self::Test(_) => Ok(ActionRunResult::new()),
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

    fn run_record_macro(
        &self,
        id: u32,
        action: &Action,
        preset_handler: &mut PresetHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .record_macro(id, Box::new(action.clone()))
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }
}
