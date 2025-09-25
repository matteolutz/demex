use std::time;

use serde::{Deserialize, Serialize};

use crate::{
    fixture::{patch::Patch, presets::preset::FixturePresetId, timing::TimingHandler},
    parser::nodes::action::{error::ActionRunError, result::ActionRunResult, Action},
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
        _issued_at: time::Instant,
        _fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        _fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        _updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _: &mut TimingHandler,
        _: &Patch,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateExecutorArgs {
    pub id: Option<u32>,
    pub sequence_id: u32,
}

impl FunctionArgs for CreateExecutorArgs {
    fn run(
        &self,
        _issued_at: time::Instant,
        _fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        _preset_handler: &mut crate::fixture::presets::PresetHandler,
        _fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _: &mut TimingHandler,
        _: &Patch,
    ) -> Result<ActionRunResult, ActionRunError> {
        updatable_handler
            .create_executor(
                self.id
                    .unwrap_or_else(|| updatable_handler.next_executor_id()),
                self.sequence_id,
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
        _issued_at: time::Instant,
        _fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        _fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        _updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _: &mut TimingHandler,
        _: &Patch,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEffectPresetArgs {
    pub id: FixturePresetId,
    pub name: Option<String>,
}

impl FunctionArgs for CreateEffectPresetArgs {
    fn run(
        &self,
        _issued_at: time::Instant,
        _fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        _fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        _updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _: &mut TimingHandler,
        _: &Patch,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .create_effect_preset(self.id, self.name.clone())
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }
}
