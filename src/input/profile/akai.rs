use crate::input::{error::DemexInputDeviceError, DemexInputDeviceProfile};

use super::DemexInputDeviceProfileType;

// **Ressources**
// https://github.com/df5602/midi-synth/blob/master/src/midi_controller.rs
// https://crates.io/crates/nusb
// https://cdn.inmusicbrands.com/akai/attachments/APC%20mini%20mk2%20-%20Communication%20Protocol%20-%20v1.0.pdf

#[derive(Debug)]
pub struct ApcMiniMk2InputDeviceProfile {}

impl ApcMiniMk2InputDeviceProfile {
    pub fn new() -> Result<Self, DemexInputDeviceError> {
        Ok(Self {})
    }
}

impl DemexInputDeviceProfile for ApcMiniMk2InputDeviceProfile {
    fn poll(
        &self,
    ) -> Result<
        Option<crate::input::message::DemexInputDeviceMessage>,
        crate::input::error::DemexInputDeviceError,
    > {
        Ok(None)
    }

    fn profile_type(&self) -> super::DemexInputDeviceProfileType {
        DemexInputDeviceProfileType::ApcMiniMk2
    }
}
