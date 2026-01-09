use std::time;

use crate::{
    command::parser::nodes::fixture_selector::FixtureSelectorContext, event::list::DemexEventList,
    input::DemexInputDeviceHandler, patch::Patch, presets::PresetHandler,
    state::fixture_state_handler::FixtureStateHandler, timing::TimingHandler,
    updatables::UpdatableHandler,
};

use super::{error::ActionRunError, result::ActionRunResult};

pub mod assign_function;
pub mod create_function;
pub mod delete_function;
pub mod go_function;
pub mod move_function;
pub mod recall_function;
pub mod record_function;
pub mod rename_function;
pub mod set_function;
pub mod stop_function;
pub mod update_function;

pub trait FunctionArgs {
    fn run(
        &self,
        issued_at: time::Instant,
        fixture_handler: &mut FixtureStateHandler,
        preset_handler: &mut PresetHandler,
        fixture_selector_context: FixtureSelectorContext,
        updatable_handler: &mut UpdatableHandler,
        input_device_handler: &mut DemexInputDeviceHandler,
        timing_handler: &mut TimingHandler,
        patch: &Patch,
        event_list: &mut DemexEventList,
    ) -> Result<ActionRunResult, ActionRunError>;
}
