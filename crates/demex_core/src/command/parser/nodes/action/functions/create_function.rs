use std::time;

use serde::{Deserialize, Serialize};

use crate::{
    command::parser::nodes::action::{Action, error::ActionRunError, result::ActionRunResult},
    event::DemexEvent,
    patch::Patch,
    pool::PoolType,
    presets::preset::FixturePresetId,
    timing::TimingHandler,
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
        _fixture_handler: &mut crate::state::fixture_state_handler::FixtureStateHandler,
        preset_handler: &mut crate::presets::PresetHandler,
        _fixture_selector_context: crate::command::parser::nodes::fixture_selector::FixtureSelectorContext,
        _updatable_handler: &mut crate::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _: &mut TimingHandler,
        _: &Patch,
    ) -> Result<
        crate::command::parser::nodes::action::result::ActionRunResult,
        crate::command::parser::nodes::action::error::ActionRunError,
    > {
        let id = self.id.unwrap_or_else(|| preset_handler.next_sequence_id());

        preset_handler
            .create_sequence(id, self.name.clone())
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::event(DemexEvent::PoolItemAdded(
            PoolType::Sequence,
            id,
        )))
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
        _fixture_handler: &mut crate::state::fixture_state_handler::FixtureStateHandler,
        _preset_handler: &mut crate::presets::PresetHandler,
        _fixture_selector_context: crate::command::parser::nodes::fixture_selector::FixtureSelectorContext,
        updatable_handler: &mut crate::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _: &mut TimingHandler,
        _: &Patch,
    ) -> Result<ActionRunResult, ActionRunError> {
        let id = self
            .id
            .unwrap_or_else(|| updatable_handler.next_executor_id());

        updatable_handler
            .create_executor(id, self.sequence_id)
            .map_err(ActionRunError::UpdatableHandlerError)?;

        Ok(ActionRunResult::event(DemexEvent::PoolItemAdded(
            PoolType::Executor,
            id,
        )))
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
        _fixture_handler: &mut crate::state::fixture_state_handler::FixtureStateHandler,
        preset_handler: &mut crate::presets::PresetHandler,
        _fixture_selector_context: crate::command::parser::nodes::fixture_selector::FixtureSelectorContext,
        _updatable_handler: &mut crate::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _: &mut TimingHandler,
        _: &Patch,
    ) -> Result<ActionRunResult, ActionRunError> {
        let id = self.id.unwrap_or_else(|| preset_handler.next_macro_id());

        preset_handler
            .create_macro(id, self.name.clone(), self.action.clone())
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::event(DemexEvent::PoolItemAdded(
            PoolType::Macro,
            id,
        )))
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
        _fixture_handler: &mut crate::state::fixture_state_handler::FixtureStateHandler,
        preset_handler: &mut crate::presets::PresetHandler,
        _fixture_selector_context: crate::command::parser::nodes::fixture_selector::FixtureSelectorContext,
        _updatable_handler: &mut crate::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _: &mut TimingHandler,
        _: &Patch,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .create_effect_preset(self.id, self.name.clone())
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::event(DemexEvent::PoolItemAdded(
            PoolType::Preset(self.id.feature_group),
            self.id.preset_id,
        )))
    }
}
