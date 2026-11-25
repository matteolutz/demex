use crate::command::parser::nodes::action::{
    ActionRunArgs, error::ActionRunError, result::ActionRunResult,
};

use super::FunctionDelegate;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedMasterTapArgs {
    pub speedmaster_id: u32,
}

impl FunctionDelegate for SpeedMasterTapArgs {
    fn run(
        &self,
        ActionRunArgs {
            timing_handler,
            issued_at,
            event_list,
            ..
        }: ActionRunArgs,
    ) -> Result<
        crate::command::parser::nodes::action::result::ActionRunResult,
        crate::command::parser::nodes::action::error::ActionRunError,
    > {
        timing_handler
            .tap_speed_master_value(self.speedmaster_id, issued_at, event_list)
            .map_err(ActionRunError::TimingHandlerError)
            .map(|_| ActionRunResult::Default)
    }
}
