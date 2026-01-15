use serde::{Deserialize, Serialize};

use crate::command::parser::nodes::{
    action::{ActionRunArgs, error::ActionRunError, result::ActionRunResult},
    object::{Object, ObjectDelegate},
};

use super::FunctionDelegate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameObjectArgs {
    pub object: Object,
    pub new_name: String,
}

impl FunctionDelegate for RenameObjectArgs {
    fn run(&self, args: ActionRunArgs) -> Result<ActionRunResult, ActionRunError> {
        self.object.clone().set(
            args.preset_handler,
            args.updatable_handler,
            args.fixture_selector_context,
            args.event_list,
            "Name".to_string(),
            self.new_name.clone(),
        )
    }
}
