use crate::command::parser::nodes::action::result::ActionRunResult;

use super::FunctionArgs;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorStompArgs {
    pub executor_id: u32,
    pub stomped: bool,
}

impl FunctionArgs for ExecutorStompArgs {
    fn run(
        &self,
        _issued_at: std::time::Instant,
        _fixture_handler: &mut crate::state::fixture_state_handler::FixtureStateHandler,
        _preset_handler: &mut crate::presets::PresetHandler,
        _fixture_selector_context: crate::command::parser::nodes::fixture_selector::FixtureSelectorContext,
        updatable_handler: &mut crate::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _timing_handler: &mut crate::timing::TimingHandler,
        _patch: &crate::patch::Patch,
        _event_list: &mut crate::event::list::DemexEventList,
    ) -> Result<
        crate::command::parser::nodes::action::result::ActionRunResult,
        crate::command::parser::nodes::action::error::ActionRunError,
    > {
        updatable_handler.executor_set_stomped(self.executor_id, self.stomped);

        Ok(ActionRunResult::Default)
    }
}
