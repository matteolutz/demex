use serde::{Deserialize, Serialize};

use crate::fixture::{
    channel::{
        value::{FixtureChannelDiscreteValue, FixtureChannelValue},
        FIXTURE_CHANNEL_COLOR_ID, FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
    },
    handler::FixtureHandler,
    presets::PresetHandler,
};

use self::{error::ActionRunError, result::ActionRunResult};

use super::{
    fixture_selector::{FixtureSelector, FixtureSelectorContext},
    object::HomeableObject,
};

pub mod error;
pub mod result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SequenceCreationMode {
    Fader,
    Button,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    SetIntensity(FixtureSelector, f32),
    SetColor(FixtureSelector, [f32; 4]),
    SetColorPreset(FixtureSelector, u32),
    SetPosition(FixtureSelector, [f32; 2]),
    SetPositionPreset(FixtureSelector, u32),
    SetChannelValue(FixtureSelector, u16, f32),
    SetChannelValuePreset(FixtureSelector, u16, u32),
    Home(HomeableObject),
    HomeAll,
    GoHome(FixtureSelector),
    GoHomeAll,
    ManSet(FixtureSelector, String, f32),
    RecordPreset(u16, FixtureSelector, Option<String>),
    RecordGroup2(FixtureSelector, Option<String>),
    RecordGroup(FixtureSelector, u32),
    RecordColor(FixtureSelector, u32),
    RecordPosition(FixtureSelector, u32),
    RecordMacro(Box<Action>, u32),
    RenameGroup(u32, String),
    RenameColorPreset(u32, String),
    RenamePositionPreset(u32, String),
    CreateSequence(SequenceCreationMode, Option<String>),
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
    ) -> Result<ActionRunResult, ActionRunError> {
        match self {
            Self::SetIntensity(fixture_selector, intensity) => self.run_set_intensity(
                preset_handler,
                fixture_handler,
                fixture_selector,
                fixture_selector_context,
                *intensity,
            ),
            Self::SetColorPreset(fixture_selector, preset_id) => self.run_set_color_preset(
                preset_handler,
                fixture_handler,
                fixture_selector,
                fixture_selector_context,
                *preset_id,
            ),
            Self::SetColor(fixture_selector, rgbw) => self.run_set_color(
                preset_handler,
                fixture_handler,
                fixture_selector,
                fixture_selector_context,
                *rgbw,
            ),
            Self::SetPositionPreset(fixture_selector, preset_id) => self.run_set_position_preset(
                preset_handler,
                fixture_handler,
                fixture_selector,
                fixture_selector_context,
                *preset_id,
            ),
            Self::SetPosition(fixture_selector, position) => self.run_set_position(
                preset_handler,
                fixture_handler,
                fixture_selector,
                fixture_selector_context,
                *position,
            ),
            Self::GoHome(fixture_selector) => self.run_go_home(
                preset_handler,
                fixture_handler,
                fixture_selector,
                fixture_selector_context,
            ),
            Self::GoHomeAll => self.run_go_home_all(fixture_handler),
            Self::ManSet(fixture_selector, channel_name, channel_value) => self.run_man_set(
                preset_handler,
                fixture_handler,
                fixture_selector,
                fixture_selector_context,
                channel_name,
                *channel_value,
            ),
            Self::RecordGroup(fixture_selector, id) => self.run_record_group(
                fixture_selector,
                fixture_selector_context,
                *id,
                preset_handler,
            ),
            Self::RenameGroup(id, new_name) => self.run_rename_group(*id, new_name, preset_handler),
            Self::RenameColorPreset(id, new_name) => {
                self.run_rename_color(*id, new_name, preset_handler)
            }
            Self::RenamePositionPreset(id, new_name) => {
                self.run_rename_position(*id, new_name, preset_handler)
            }
            Self::RecordColor(fixture_selector, id) => self.run_record_color(
                fixture_selector,
                fixture_selector_context,
                *id,
                preset_handler,
                fixture_handler,
            ),
            Self::RecordPosition(fixture_selector, id) => self.run_record_position(
                fixture_selector,
                fixture_selector_context,
                *id,
                preset_handler,
                fixture_handler,
            ),
            Self::RecordMacro(action, id) => self.run_record_macro(*id, action, preset_handler),
            Self::ClearAll => Ok(ActionRunResult::new()),
            Self::FixtureSelector(_) => Ok(ActionRunResult::new()),
            Self::Test(_) => Ok(ActionRunResult::new()),
            _ => todo!(),
        }
    }

    fn run_go_home_all(
        &self,
        fixture_handler: &mut FixtureHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        fixture_handler
            .home_all()
            .map_err(ActionRunError::FixtureHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_go_home(
        &self,
        preset_handler: &PresetHandler,
        fixture_handler: &mut FixtureHandler,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
    ) -> Result<ActionRunResult, ActionRunError> {
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

    fn run_set_intensity(
        &self,
        preset_handler: &PresetHandler,
        fixture_handler: &mut FixtureHandler,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        intensity: f32,
    ) -> Result<ActionRunResult, ActionRunError> {
        let fixtures = fixture_selector
            .get_fixtures(preset_handler, fixture_selector_context)
            .map_err(ActionRunError::FixtureSelectorError)?;

        for fixture in fixtures {
            if let Some(f) = fixture_handler.fixture(fixture) {
                f.set_intensity(FixtureChannelValue::Discrete(
                    FixtureChannelDiscreteValue::Single(intensity / 100.0),
                ))
                .map_err(ActionRunError::FixtureError)?;
            }
        }

        Ok(ActionRunResult::new())
    }

    fn run_set_color_preset(
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
            .get_preset(preset_id, FIXTURE_CHANNEL_COLOR_ID)
            .map_err(ActionRunError::PresetHandlerError)?;

        for fixture in fixtures {
            if let Some(f) = fixture_handler.fixture(fixture) {
                f.set_color(FixtureChannelValue::Preset(preset.id()))
                    .map_err(ActionRunError::FixtureError)?;
            }
        }

        Ok(ActionRunResult::new())
    }

    fn run_set_color(
        &self,
        preset_handler: &PresetHandler,
        fixture_handler: &mut FixtureHandler,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        color: [f32; 4],
    ) -> Result<ActionRunResult, ActionRunError> {
        let fixtures = fixture_selector
            .get_fixtures(preset_handler, fixture_selector_context)
            .map_err(ActionRunError::FixtureSelectorError)?;

        for fixture in fixtures {
            if let Some(f) = fixture_handler.fixture(fixture) {
                f.set_color(FixtureChannelValue::Discrete(
                    FixtureChannelDiscreteValue::Quadruple(color),
                ))
                .map_err(ActionRunError::FixtureError)?;
            }
        }

        Ok(ActionRunResult::new())
    }

    fn run_set_position_preset(
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
            .get_preset(preset_id, FIXTURE_CHANNEL_POSITION_PAN_TILT_ID)
            .map_err(ActionRunError::PresetHandlerError)?;

        for fixture in fixtures {
            if let Some(f) = fixture_handler.fixture(fixture) {
                f.set_position_pan_tilt(FixtureChannelValue::Preset(preset.id()))
                    .map_err(ActionRunError::FixtureError)?;
            }
        }

        Ok(ActionRunResult::new())
    }

    fn run_set_position(
        &self,
        preset_handler: &PresetHandler,
        fixture_handler: &mut FixtureHandler,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        position: [f32; 2],
    ) -> Result<ActionRunResult, ActionRunError> {
        let fixtures = fixture_selector
            .get_fixtures(preset_handler, fixture_selector_context)
            .map_err(ActionRunError::FixtureSelectorError)?;

        for fixture in fixtures {
            if let Some(f) = fixture_handler.fixture(fixture) {
                f.set_position_pan_tilt(FixtureChannelValue::Discrete(
                    FixtureChannelDiscreteValue::Pair(position),
                ))
                .map_err(ActionRunError::FixtureError)?;
            }
        }

        Ok(ActionRunResult::new())
    }

    fn run_man_set(
        &self,
        preset_handler: &PresetHandler,
        fixture_handler: &mut FixtureHandler,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        channel_name: &str,
        channel_value: f32,
    ) -> Result<ActionRunResult, ActionRunError> {
        let fixtures = fixture_selector
            .get_fixtures(preset_handler, fixture_selector_context)
            .map_err(ActionRunError::FixtureSelectorError)?;

        for fixture in fixtures {
            if let Some(f) = fixture_handler.fixture(fixture) {
                f.set_mainenance(
                    channel_name,
                    FixtureChannelValue::Discrete(FixtureChannelDiscreteValue::Single(
                        channel_value / 100.0,
                    )),
                )
                .map_err(ActionRunError::FixtureError)?;
            }
        }

        Ok(ActionRunResult::new())
    }

    fn run_record_group(
        &self,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        id: u32,
        preset_handler: &mut PresetHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .record_group(
                fixture_selector
                    .flatten(preset_handler, fixture_selector_context)
                    .map_err(ActionRunError::FixtureSelectorError)?,
                id,
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_rename_group(
        &self,
        id: u32,
        new_name: &str,
        preset_handler: &mut PresetHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .rename_group(id, new_name.to_owned())
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_rename_color(
        &self,
        id: u32,
        new_name: &str,
        preset_handler: &mut PresetHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .rename_preset(id, FIXTURE_CHANNEL_COLOR_ID, new_name.to_owned())
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_rename_position(
        &self,
        id: u32,
        new_name: &str,
        preset_handler: &mut PresetHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .rename_preset(
                id,
                FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
                new_name.to_owned(),
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_record_color(
        &self,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        id: u32,
        preset_handler: &mut PresetHandler,
        fixture_handler: &mut FixtureHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .record_preset(
                fixture_selector,
                fixture_selector_context,
                id,
                fixture_handler,
                FIXTURE_CHANNEL_COLOR_ID,
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_record_position(
        &self,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        id: u32,
        preset_handler: &mut PresetHandler,
        fixture_handler: &mut FixtureHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .record_preset(
                fixture_selector,
                fixture_selector_context,
                id,
                fixture_handler,
                FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
            )
            .map_err(ActionRunError::PresetHandlerError)?;

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
