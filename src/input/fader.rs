use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::{handler::FixtureHandler, updatables::UpdatableHandler};

use super::error::DemexInputDeviceError;

#[derive(Debug, Serialize, Deserialize, EguiProbe, Default, Clone)]
pub struct DemexInputFader {
    fader_id: u32,
}

impl DemexInputFader {
    pub fn new(fader_id: u32) -> Self {
        Self { fader_id }
    }

    pub fn handle_change(
        &self,
        value: f32,
        fixture_handler: &mut FixtureHandler,
        updatable_handler: &mut UpdatableHandler,
    ) -> Result<(), DemexInputDeviceError> {
        let fader = updatable_handler
            .fader_mut(self.fader_id)
            .map_err(DemexInputDeviceError::UpdatableHandlerError)?;

        if !fader.is_active() {
            fader.activate(fixture_handler);
        }

        fader.set_value(value);
        Ok(())
    }
}
