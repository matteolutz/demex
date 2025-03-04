use serde::{Deserialize, Serialize};

use crate::parser::nodes::{
    action::{error::ActionRunError, result::ActionRunResult, Action},
    fixture_selector::FixtureSelector,
};

use super::FunctionArgs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSequenceArgs {
    pub id: Option<u32>,
    pub name: Option<String>,
}

impl FunctionArgs for CreateSequenceArgs {
    fn run(
        &self,
        _fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        _fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        _updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
    ) -> Result<
        crate::parser::nodes::action::result::ActionRunResult,
        crate::parser::nodes::action::error::ActionRunError,
    > {
        preset_handler
            .create_sequence(
                self.id.unwrap_or_else(|| preset_handler.next_sequence_id()),
                self.name.clone(),
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CreateExecutorArgsCreationMode {
    Sequence(u32),
    Effect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateExecutorArgs {
    pub id: Option<u32>,
    pub creation_mode: CreateExecutorArgsCreationMode,
    pub fixture_selector: FixtureSelector,
    pub name: Option<String>,
}

impl FunctionArgs for CreateExecutorArgs {
    fn run(
        &self,
        _fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        let selection = self
            .fixture_selector
            .get_selection(preset_handler, fixture_selector_context)
            .map_err(ActionRunError::FixtureSelectorError)?;

        updatable_handler
            .create_executor(
                self.id
                    .unwrap_or_else(|| updatable_handler.next_executor_id()),
                self.name.clone(),
                &self.creation_mode,
                selection,
            )
            .map_err(ActionRunError::UpdatableHandlerError)?;

        Ok(ActionRunResult::new())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CreateFaderArgsCreationMode {
    Submaster(FixtureSelector),
    Sequence(u32, FixtureSelector),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFaderArgs {
    pub id: Option<u32>,
    pub creation_mode: CreateFaderArgsCreationMode,
    pub name: Option<String>,
}

impl FunctionArgs for CreateFaderArgs {
    fn run(
        &self,
        _fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        updatable_handler
            .create_fader(
                self.id.unwrap_or_else(|| updatable_handler.next_fader_id()),
                &self.creation_mode,
                self.name.clone(),
                preset_handler,
                fixture_selector_context,
            )
            .map_err(ActionRunError::UpdatableHandlerError)?;

        Ok(ActionRunResult::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMacroArgs {
    pub id: Option<u32>,
    pub action: Box<Action>,
    pub name: Option<String>,
}

impl FunctionArgs for CreateMacroArgs {
    fn run(
        &self,
        _fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        _fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        _updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .create_macro(
                self.id.unwrap_or_else(|| preset_handler.next_macro_id()),
                self.name.clone(),
                self.action.clone(),
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }
}
