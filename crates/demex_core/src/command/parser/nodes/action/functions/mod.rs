use crate::command::parser::nodes::action::ActionRunArgs;

use super::{error::ActionRunError, result::ActionRunResult};

pub mod assign_function;
pub mod create_function;
pub mod delete_function;
pub mod effect_function;
pub mod go_function;
pub mod move_function;
pub mod patch_function;
pub mod recall_function;
pub mod record_function;
pub mod rename_function;
pub mod set_function;
pub mod speedmaster_functions;
pub mod start_function;
pub mod stomp_function;
pub mod stop_function;
pub mod update_function;

pub trait FunctionDelegate {
    fn run(&self, args: ActionRunArgs) -> Result<ActionRunResult, ActionRunError>;
}
