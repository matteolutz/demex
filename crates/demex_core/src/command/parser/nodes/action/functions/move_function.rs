use serde::{Deserialize, Serialize};

use crate::{
    command::parser::nodes::action::{
        error::ActionRunError, functions::FunctionArgs, result::ActionRunResult,
    },
    presets::preset::FixturePresetId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveArgs {
    pub preset_to_move: FixturePresetId,
    pub target_preset: FixturePresetId,
}

impl FunctionArgs for MoveArgs {
    fn run(
        &self,
        _: std::time::Instant,
        _: &mut crate::state::fixture_state_handler::FixtureStateHandler,
        preset_handler: &mut crate::presets::PresetHandler,
        _: crate::command::parser::nodes::fixture_selector::FixtureSelectorContext,
        _: &mut crate::updatables::UpdatableHandler,
        _: &mut crate::input::DemexInputDeviceHandler,
        _: &mut crate::timing::TimingHandler,
        _: &crate::patch::Patch,
        event_list: &mut crate::event::list::DemexEventList,
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
