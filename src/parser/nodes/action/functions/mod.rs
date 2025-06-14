use std::time;

use crate::{
    fixture::{
        handler::FixtureHandler, patch::Patch, presets::PresetHandler, timing::TimingHandler,
        updatables::UpdatableHandler,
    },
    input::DemexInputDeviceHandler,
    parser::nodes::fixture_selector::FixtureSelectorContext,
};

use super::{error::ActionRunError, result::ActionRunResult};

pub mod assign_function;
pub mod create_function;
pub mod delete_function;
pub mod go_function;
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
        fixture_handler: &mut FixtureHandler,
        preset_handler: &mut PresetHandler,
        fixture_selector_context: FixtureSelectorContext,
        updatable_handler: &mut UpdatableHandler,
        input_device_handler: &mut DemexInputDeviceHandler,
        timing_handler: &mut TimingHandler,
        patch: &Patch,
    ) -> Result<ActionRunResult, ActionRunError>;
}
