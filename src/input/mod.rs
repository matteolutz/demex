use device::{DemexInputDevice, DemexInputDeviceConfig};
use error::DemexInputDeviceError;
use message::DemexInputDeviceMessage;
use profile::DemexInputDeviceProfileType;

use crate::{
    fixture::{handler::FixtureHandler, presets::PresetHandler, updatables::UpdatableHandler},
    parser::nodes::fixture_selector::FixtureSelectorContext,
};

pub mod button;
pub mod device;
pub mod error;
pub mod fader;
pub mod message;
pub mod midi;
pub mod profile;

pub trait DemexInputDeviceProfile: std::fmt::Debug {
    fn update_out(
        &mut self,
        device_config: &DemexInputDeviceConfig,
        updatable_handler: &UpdatableHandler,
    ) -> Result<(), DemexInputDeviceError>;
    fn poll(&self) -> Result<Vec<DemexInputDeviceMessage>, DemexInputDeviceError>;
    fn profile_type(&self) -> DemexInputDeviceProfileType;
}

#[derive(Default, Debug)]
pub struct DemexInputDeviceHandler {
    devices: Vec<DemexInputDevice>,
}

impl DemexInputDeviceHandler {
    pub fn new(devices: Vec<DemexInputDevice>) -> Self {
        Self { devices }
    }

    pub fn devices(&self) -> &Vec<DemexInputDevice> {
        &self.devices
    }

    pub fn update(
        &mut self,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
        updatable_handler: &mut UpdatableHandler,
        fixture_selector_context: FixtureSelectorContext,
    ) -> Result<(), DemexInputDeviceError> {
        for device in &self.devices {
            for device_message in device.profile().poll()? {
                match device_message {
                    DemexInputDeviceMessage::ButtonPressed(button_id) => {
                        let button = device
                            .config()
                            .buttons()
                            .get(&button_id)
                            .ok_or(DemexInputDeviceError::ButtonNotFound(button_id))?;
                        button.handle_press(
                            fixture_handler,
                            preset_handler,
                            updatable_handler,
                            fixture_selector_context.clone(),
                        )?;
                    }
                    DemexInputDeviceMessage::ButtonReleased(button_id) => {
                        let button = device
                            .config()
                            .buttons()
                            .get(&button_id)
                            .ok_or(DemexInputDeviceError::ButtonNotFound(button_id))?;
                        button.handle_release(
                            fixture_handler,
                            preset_handler,
                            updatable_handler,
                        )?;
                    }
                    DemexInputDeviceMessage::FaderValueChanged(fader_id, value) => {
                        let fader = device
                            .config()
                            .faders()
                            .get(&fader_id)
                            .ok_or(DemexInputDeviceError::ButtonNotFound(fader_id))?;
                        fader.handle_change(value, fixture_handler, updatable_handler)?;
                    }
                }
            }
        }

        for device in &mut self.devices {
            device
                .profile
                .update_out(&device.config, updatable_handler)?;
        }

        Ok(())
    }
}
