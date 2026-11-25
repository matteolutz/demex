use serde::{Deserialize, Serialize};

use crate::{
    command::parser::nodes::action::{
        Action, ActionRunArgs, error::ActionRunError, result::ActionRunResult,
    },
    presets::preset::FixturePresetId,
};

use super::FunctionDelegate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSequenceArgs {
    pub id: Option<u32>,
    pub name: Option<String>,
}

impl FunctionDelegate for CreateSequenceArgs {
    fn run(
        &self,
        ActionRunArgs {
            preset_handler,
            event_list,
            ..
        }: ActionRunArgs,
    ) -> Result<
        crate::command::parser::nodes::action::result::ActionRunResult,
        crate::command::parser::nodes::action::error::ActionRunError,
    > {
        let id = self.id.unwrap_or_else(|| preset_handler.next_sequence_id());

        preset_handler
            .create_sequence(id, self.name.clone(), event_list)
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::Default)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateExecutorArgs {
    pub id: Option<u32>,
    pub sequence_id: u32,
}

impl FunctionDelegate for CreateExecutorArgs {
    fn run(
        &self,
        ActionRunArgs {
            updatable_handler,
            event_list,
            ..
        }: ActionRunArgs,
    ) -> Result<ActionRunResult, ActionRunError> {
        let id = self
            .id
            .unwrap_or_else(|| updatable_handler.next_executor_id());

        updatable_handler
            .create_executor(id, self.sequence_id, event_list)
            .map_err(ActionRunError::UpdatableHandlerError)?;

        Ok(ActionRunResult::Default)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMacroArgs {
    pub id: Option<u32>,
    pub action: Box<Action>,
    pub name: Option<String>,
}

impl FunctionDelegate for CreateMacroArgs {
    fn run(
        &self,
        ActionRunArgs {
            preset_handler,
            event_list,
            ..
        }: ActionRunArgs,
    ) -> Result<ActionRunResult, ActionRunError> {
        let id = self.id.unwrap_or_else(|| preset_handler.next_macro_id());

        preset_handler
            .create_macro(id, self.name.clone(), self.action.clone(), event_list)
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::Default)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEffectPresetArgs {
    pub id: FixturePresetId,
    pub name: Option<String>,
}

impl FunctionDelegate for CreateEffectPresetArgs {
    fn run(
        &self,
        ActionRunArgs {
            preset_handler,
            event_list,
            ..
        }: ActionRunArgs,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .create_effect_preset(self.id, self.name.clone(), event_list)
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::Default)
    }
}
