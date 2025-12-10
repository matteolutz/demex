use serde::{Deserialize, Serialize};

use crate::{
    command::parser::nodes::action::{
        error::ActionRunError, functions::FunctionArgs, result::ActionRunResult,
    },
    event::DemexEvent,
    pool::PoolType,
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
    ) -> Result<
        crate::command::parser::nodes::action::result::ActionRunResult,
        crate::command::parser::nodes::action::error::ActionRunError,
    > {
        preset_handler
            .move_preset(self.preset_to_move, self.target_preset)
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::event(DemexEvent::PoolItemMoved {
            // the PresetHandler will make sure that the pool type matches
            pool_type: PoolType::Preset(self.preset_to_move.feature_group),
            from_id: self.preset_to_move.preset_id,
            to_id: self.target_preset.preset_id,
        }))
    }
}
