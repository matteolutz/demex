use crate::command::parser::nodes::action::{ActionRunArgs, result::ActionRunResult};

use super::FunctionDelegate;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorStompArgs {
    pub executor_id: u32,
    pub stomped: bool,
}

impl FunctionDelegate for ExecutorStompArgs {
    fn run(
        &self,
        ActionRunArgs {
            updatable_handler, ..
        }: ActionRunArgs,
    ) -> Result<
        crate::command::parser::nodes::action::result::ActionRunResult,
        crate::command::parser::nodes::action::error::ActionRunError,
    > {
        updatable_handler.executor_set_stomped(self.executor_id, self.stomped);

        Ok(ActionRunResult::Default)
    }
}
