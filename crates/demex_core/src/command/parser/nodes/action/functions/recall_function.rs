use serde::{Deserialize, Serialize};

use crate::{
    command::parser::nodes::action::{
        ActionRunArgs, error::ActionRunError, result::ActionRunResult,
    },
    presets::error::PresetHandlerError,
    sequence::cue::CueIdx,
};

use super::FunctionDelegate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallSequenceCueArgs {
    pub sequence_id: u32,
    pub cue_idx: CueIdx,
}

impl FunctionDelegate for RecallSequenceCueArgs {
    fn run(
        &self,
        ActionRunArgs {
            preset_handler,
            patch,
            fixture_handler,
            ..
        }: ActionRunArgs,
    ) -> Result<
        crate::command::parser::nodes::action::result::ActionRunResult,
        crate::command::parser::nodes::action::error::ActionRunError,
    > {
        let sequence = preset_handler
            .get_sequence(self.sequence_id)
            .map_err(ActionRunError::PresetHandlerError)?;

        let cue = sequence
            .find_cue(self.cue_idx)
            .ok_or(ActionRunError::PresetHandlerError(
                PresetHandlerError::CueNotFound(self.sequence_id, self.cue_idx),
            ))?;

        cue.recall(patch, fixture_handler);

        Ok(ActionRunResult::new())
    }
}
