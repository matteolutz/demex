use device::{DemexInputDevice, DemexInputDeviceConfig};
use error::DemexInputDeviceError;
use message::DemexInputDeviceMessage;
use profile::DemexInputDeviceProfileType;

use crate::{
    fixture::{handler::FixtureHandler, presets::PresetHandler, updatables::UpdatableHandler},
    parser::nodes::{
        action::Action,
        fixture_selector::{FixtureSelector, FixtureSelectorContext},
    },
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
        global_fixture_selector: &Option<FixtureSelector>,
    ) -> Result<(), DemexInputDeviceError>;
    fn poll(&self) -> Result<Vec<DemexInputDeviceMessage>, DemexInputDeviceError>;
    fn profile_type(&self) -> DemexInputDeviceProfileType;
}

#[derive(Default, Debug)]
pub struct DemexInputDeviceHandler {
    initial_configs: Vec<DemexInputDeviceConfig>,
    devices: Vec<DemexInputDevice>,
}

impl DemexInputDeviceHandler {
    pub fn new(
        initial_configs: Vec<DemexInputDeviceConfig>,
        devices: Vec<DemexInputDevice>,
    ) -> Self {
        Self {
            initial_configs,
            devices,
        }
    }

    pub fn devices(&self) -> &Vec<DemexInputDevice> {
        &self.devices
    }

    pub fn initial_configs(&self) -> &Vec<DemexInputDeviceConfig> {
        &self.initial_configs
    }

    pub fn update(
        &mut self,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
        updatable_handler: &mut UpdatableHandler,
        fixture_selector_context: FixtureSelectorContext,
        macro_exec_cue: &mut Vec<Action>,
        global_fixture_selector: &mut Option<FixtureSelector>,
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
                            macro_exec_cue,
                            global_fixture_selector,
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
            device.profile.update_out(
                &device.config,
                updatable_handler,
                global_fixture_selector,
            )?;
        }

        Ok(())
    }
}
