use crate::parser::nodes::action::{error::ActionRunError, result::ActionRunResult};

use super::FunctionArgs;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorGoArgs {
    pub executor_id: u32,
}

impl FunctionArgs for ExecutorGoArgs {
    fn run(
        &self,
        issued_at: std::time::Instant,
        fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        _fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _timing_handler: &mut crate::fixture::timing::TimingHandler,
        _patch: &crate::fixture::patch::Patch,
    ) -> Result<
        crate::parser::nodes::action::result::ActionRunResult,
        crate::parser::nodes::action::error::ActionRunError,
    > {
        updatable_handler
            .start_or_next_executor(
                self.executor_id,
                fixture_handler,
                preset_handler,
                issued_at.elapsed().as_secs_f32(),
            )
            .map_err(ActionRunError::UpdatableHandlerError)
            .map(|_| ActionRunResult::new())
    }
}
