use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{
    button::DemexInputButton,
    fader::DemexInputFader,
    profile::{
        akai::ApcMiniMk2InputDeviceProfile, midi_timecode::MidiTimecodeProfile,
        DemexInputDeviceProfileType,
    },
    DemexInputDeviceProfile,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
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

    pub fn buttons_mut(&mut self) -> &mut HashMap<u32, DemexInputButton> {
        &mut self.buttons
    }

    pub fn faders(&self) -> &HashMap<u32, DemexInputFader> {
        &self.faders
    }

    pub fn faders_mut(&mut self) -> &mut HashMap<u32, DemexInputFader> {
        &mut self.faders
    }

    pub fn profile_type(&self) -> &DemexInputDeviceProfileType {
        &self.profile_type
    }
}

#[derive(Debug)]
pub struct DemexInputDevice {
    pub(crate) profile: Box<dyn DemexInputDeviceProfile>,
    pub(crate) config: DemexInputDeviceConfig,
}

impl DemexInputDevice {
    pub fn profile(&self) -> &dyn DemexInputDeviceProfile {
        self.profile.as_ref()
    }

    pub fn config(&self) -> &DemexInputDeviceConfig {
        &self.config
    }
}

impl From<DemexInputDeviceConfig> for DemexInputDevice {
    fn from(value: DemexInputDeviceConfig) -> Self {
        let profile: Box<dyn DemexInputDeviceProfile> = match value.profile_type {
            DemexInputDeviceProfileType::ApcMiniMk2 { ref apc_midi } => {
                Box::new(ApcMiniMk2InputDeviceProfile::new(apc_midi.clone()))
            }
            DemexInputDeviceProfileType::MidiTimecode { ref midi_in_device } => {
                Box::new(MidiTimecodeProfile::new(midi_in_device.clone()))
            }
        };

        DemexInputDevice {
            config: value,
            profile,
        }
    }
}
