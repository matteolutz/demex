use serde::{Deserialize, Serialize};

use crate::{
    command::parser::nodes::action::{
        ActionRunArgs, error::ActionRunError, functions::FunctionDelegate, result::ActionRunResult,
    },
    presets::preset::FixturePresetId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveArgs {
    pub preset_to_move: FixturePresetId,
    pub target_preset: FixturePresetId,
}

impl FunctionDelegate for MoveArgs {
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
        preset_handler
            .move_preset(self.preset_to_move, self.target_preset, event_list)
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::Default)
    }
}
