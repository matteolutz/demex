use egui_probe::EguiProbe;
use error::DemexInputDeviceError;
use message::DemexInputDeviceMessage;
use profile::{akai::ApcMiniMk2InputDeviceProfile, DemexInputDeviceProfileType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::fixture::{
    handler::FixtureHandler, presets::PresetHandler, updatables::UpdatableHandler,
};

use button::DemexInputButton;

pub mod button;
pub mod error;
pub mod message;
pub mod midi;
pub mod profile;

pub trait DemexInputDeviceProfile: std::fmt::Debug {
    fn poll(&self) -> Result<Option<DemexInputDeviceMessage>, DemexInputDeviceError>;
    fn profile_type(&self) -> DemexInputDeviceProfileType;
}

#[derive(Debug, Serialize, Deserialize, EguiProbe)]
pub struct DemexInputDeviceConfig {
    buttons: HashMap<u32, DemexInputButton>,
    profile_type: DemexInputDeviceProfileType,
}

impl DemexInputDeviceConfig {
    pub fn new(
        buttons: HashMap<u32, DemexInputButton>,
        profile_type: DemexInputDeviceProfileType,
    ) -> Self {
        Self {
            buttons,
            profile_type,
        }
    }
}

#[derive(Debug)]
pub struct DemexInputDevice {
    profile: Box<dyn DemexInputDeviceProfile>,
    config: DemexInputDeviceConfig,
}

impl DemexInputDevice {
    pub fn profile(&self) -> &dyn DemexInputDeviceProfile {
        self.profile.as_ref()
    }

    pub fn config(&self) -> &DemexInputDeviceConfig {
        &self.config
    }
}

impl TryFrom<DemexInputDeviceConfig> for DemexInputDevice {
    type Error = DemexInputDeviceError;

    fn try_from(value: DemexInputDeviceConfig) -> Result<Self, Self::Error> {
        let profile = match value.profile_type {
            DemexInputDeviceProfileType::ApcMiniMk2 => ApcMiniMk2InputDeviceProfile::new()?,
        };

        Ok(DemexInputDevice {
            config: value,
            profile: Box::new(profile),
        })
    }
}

#[derive(Default, Debug)]
pub struct DemexInputDeviceHandler {
    devices: Vec<DemexInputDevice>,
}

impl DemexInputDeviceHandler {
    pub fn new(devices: Vec<DemexInputDevice>) -> Self {
        Self { devices }
    }

    pub fn update(
        &self,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
        updatable_handler: &mut UpdatableHandler,
    ) -> Result<(), DemexInputDeviceError> {
        for device in &self.devices {
            if let Some(device_message) = device.profile.poll()? {
                match device_message {
                    DemexInputDeviceMessage::ButtonPressed(button_id) => {
                        let button = device
                            .config
                            .buttons
                            .get(&button_id)
                            .ok_or(DemexInputDeviceError::ButtonNotFound(button_id))?;
                        button.handle_press(fixture_handler, preset_handler, updatable_handler)?;
                    }
                    DemexInputDeviceMessage::ButtonReleased(button_id) => {
                        let button = device
                            .config
                            .buttons
                            .get(&button_id)
                            .ok_or(DemexInputDeviceError::ButtonNotFound(button_id))?;
                        button.handle_release(
                            fixture_handler,
                            preset_handler,
                            updatable_handler,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }
}
