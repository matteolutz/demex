use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::{
    handler::FixtureHandler, presets::PresetHandler, updatables::UpdatableHandler,
};

use super::error::DemexInputDeviceError;

#[derive(Debug, Serialize, Deserialize, EguiProbe, Default)]
pub enum DemexInputButton {
    ExecutorStartAndNext(u32),
    ExecutorStop(u32),
    ExecutorFlash(u32),

    #[default]
    Unused,
}

impl DemexInputButton {
    pub fn handle_press(
        &self,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
        updatable_handler: &mut UpdatableHandler,
    ) -> Result<(), DemexInputDeviceError> {
        match self {
            Self::ExecutorFlash(executor_id) => updatable_handler
                .start_executor(*executor_id, fixture_handler)
                .map_err(DemexInputDeviceError::UpdatableHandlerError)?,
            Self::ExecutorStartAndNext(executor_id) => updatable_handler
                .start_or_next_executor(*executor_id, fixture_handler, preset_handler)
                .map_err(DemexInputDeviceError::UpdatableHandlerError)?,
            Self::ExecutorStop(executor_id) => updatable_handler
                .stop_executor(*executor_id, fixture_handler)
                .map_err(DemexInputDeviceError::UpdatableHandlerError)?,
            Self::Unused => {}
        }
        Ok(())
    }

    pub fn handle_release(
        &self,
        fixture_handler: &mut FixtureHandler,
        _preset_handler: &PresetHandler,
        updatable_handler: &mut UpdatableHandler,
    ) -> Result<(), DemexInputDeviceError> {
        match self {
            Self::ExecutorFlash(executor_id) => updatable_handler
                .stop_executor(*executor_id, fixture_handler)
                .map_err(DemexInputDeviceError::UpdatableHandlerError)?,
            _ => {}
        }
        Ok(())
    }
}
