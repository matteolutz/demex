use crate::{
    fixture::{handler::FixtureHandler, presets::PresetHandler, updatables::UpdatableHandler},
    input::DemexInputDeviceHandler,
    parser::nodes::fixture_selector::FixtureSelectorContext,
};

use super::{error::ActionRunError, result::ActionRunResult};

pub mod delete_function;
pub mod record_function;
pub mod rename_function;
pub mod set_function;

pub trait ActionFunction {
    fn run(
        &self,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &mut PresetHandler,
        fixture_selector_context: FixtureSelectorContext,
        updatable_handler: &mut UpdatableHandler,
        input_device_handler: &mut DemexInputDeviceHandler,
    ) -> Result<ActionRunResult, ActionRunError>;
}
