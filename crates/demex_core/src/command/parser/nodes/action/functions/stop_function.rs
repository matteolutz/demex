use crate::command::parser::nodes::action::{
    ActionRunArgs, error::ActionRunError, result::ActionRunResult,
};

use super::FunctionDelegate;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorStopArgs {
    pub executor_id: u32,
}

impl FunctionDelegate for ExecutorStopArgs {
    fn run(
        &self,
        args: ActionRunArgs,
    ) -> Result<
        crate::command::parser::nodes::action::result::ActionRunResult,
        crate::command::parser::nodes::action::error::ActionRunError,
    > {
        args.updatable_handler
            .stop_executor(
                self.executor_id,
                args.fixture_handler,
                args.preset_handler,
                args.event_list,
            )
            .map_err(ActionRunError::UpdatableHandlerError)
            .map(|_| ActionRunResult::Default)
    }
}
