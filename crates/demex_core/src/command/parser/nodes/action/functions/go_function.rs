use crate::command::parser::nodes::action::{
    ActionRunArgs, error::ActionRunError, result::ActionRunResult,
};

use super::FunctionDelegate;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorGoArgs {
    pub executor_id: u32,
}

impl FunctionDelegate for ExecutorGoArgs {
    fn run(
        &self,
        args: ActionRunArgs,
    ) -> Result<
        crate::command::parser::nodes::action::result::ActionRunResult,
        crate::command::parser::nodes::action::error::ActionRunError,
    > {
        args.updatable_handler
            .executor_go(
                self.executor_id,
                args.fixture_handler,
                args.preset_handler,
                args.issued_at.elapsed().as_secs_f32(),
                args.event_list,
            )
            .map_err(ActionRunError::UpdatableHandlerError)
            .map(|_| ActionRunResult::Default)
    }
}
