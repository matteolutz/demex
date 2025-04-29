use serde::{Deserialize, Serialize};

use crate::{
    fixture::{patch::Patch, presets::error::PresetHandlerError, sequence::cue::CueIdx},
    parser::nodes::action::{error::ActionRunError, result::ActionRunResult},
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
        fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        _fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        _updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _timing_handler: &mut crate::fixture::timing::TimingHandler,
        patch: &Patch,
    ) -> Result<
        crate::parser::nodes::action::result::ActionRunResult,
        crate::parser::nodes::action::error::ActionRunError,
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
