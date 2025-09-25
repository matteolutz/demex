use std::time;

use serde::{Deserialize, Serialize};

use crate::{
    command::parser::nodes::action::{error::ActionRunError, result::ActionRunResult},
    patch::Patch,
    presets::error::PresetHandlerError,
    sequence::cue::CueIdx,
};

use super::FunctionArgs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallSequenceCueArgs {
    pub sequence_id: u32,
    pub cue_idx: CueIdx,
}

impl FunctionArgs for RecallSequenceCueArgs {
    fn run(
        &self,
        _issued_at: time::Instant,
        fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::presets::PresetHandler,
        _fixture_selector_context: crate::command::parser::nodes::fixture_selector::FixtureSelectorContext,
        _updatable_handler: &mut crate::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _timing_handler: &mut crate::timing::TimingHandler,
        patch: &Patch,
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

        cue.recall(patch.fixture_types(), fixture_handler);

        Ok(ActionRunResult::new())
    }
}
