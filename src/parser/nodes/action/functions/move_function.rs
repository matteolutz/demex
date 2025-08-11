use serde::{Deserialize, Serialize};

use crate::{
    fixture::presets::preset::FixturePresetId,
    parser::nodes::action::{
        error::ActionRunError, functions::FunctionArgs, result::ActionRunResult,
    },
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
        _: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        _: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        _: &mut crate::fixture::updatables::UpdatableHandler,
        _: &mut crate::input::DemexInputDeviceHandler,
        _: &mut crate::fixture::timing::TimingHandler,
        _: &crate::fixture::patch::Patch,
    ) -> Result<
        crate::parser::nodes::action::result::ActionRunResult,
        crate::parser::nodes::action::error::ActionRunError,
    > {
        preset_handler
            .move_preset(self.preset_to_move, self.target_preset)
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }
}
