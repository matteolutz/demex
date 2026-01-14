use crate::command::parser::nodes::action::{error::ActionRunError, result::ActionRunResult};

use super::FunctionArgs;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedMasterTapArgs {
    pub speedmaster_id: u32,
}

impl FunctionArgs for SpeedMasterTapArgs {
    fn run(
        &self,
        issued_at: std::time::Instant,
        _fixture_handler: &mut crate::state::fixture_state_handler::FixtureStateHandler,
        _preset_handler: &mut crate::presets::PresetHandler,
        _fixture_selector_context: crate::command::parser::nodes::fixture_selector::FixtureSelectorContext,
        _updatable_handler: &mut crate::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        timing_handler: &mut crate::timing::TimingHandler,
        _patch: &crate::patch::Patch,
        _event_list: &mut crate::event::list::DemexEventList,
    ) -> Result<
        crate::command::parser::nodes::action::result::ActionRunResult,
        crate::command::parser::nodes::action::error::ActionRunError,
    > {
        timing_handler
            .tap_speed_master_value(self.speedmaster_id, issued_at)
            .map_err(ActionRunError::TimingHandlerError)
            .map(|_| ActionRunResult::Default)
    }
}
