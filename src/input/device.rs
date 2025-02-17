use std::collections::HashMap;

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use super::{
    button::DemexInputButton,
    error::DemexInputDeviceError,
    fader::DemexInputFader,
    profile::{akai::ApcMiniMk2InputDeviceProfile, DemexInputDeviceProfileType},
    DemexInputDeviceProfile,
};

#[derive(Debug, Serialize, Deserialize, EguiProbe, Clone)]
pub struct DemexInputDeviceConfig {
    buttons: HashMap<u32, DemexInputButton>,
    faders: HashMap<u32, DemexInputFader>,
    profile_type: DemexInputDeviceProfileType,
}

impl DemexInputDeviceConfig {
    pub fn new(
        buttons: HashMap<u32, DemexInputButton>,
        faders: HashMap<u32, DemexInputFader>,
        profile_type: DemexInputDeviceProfileType,
    ) -> Self {
        Self {
            buttons,
            faders,
            profile_type,
        }
    }

    pub fn buttons(&self) -> &HashMap<u32, DemexInputButton> {
        &self.buttons
    }

    pub fn faders(&self) -> &HashMap<u32, DemexInputFader> {
        &self.faders
    }

    pub fn profile_type(&self) -> DemexInputDeviceProfileType {
        self.profile_type
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
