use std::sync::{mpsc, Arc};

use parking_lot::Mutex;

use crate::input::{
    error::DemexInputDeviceError,
    midi::{usb::UsbMidiDevice, MidiMessage},
    DemexInputDeviceProfile,
};

use super::DemexInputDeviceProfileType;

// **Ressources**
// https://github.com/df5602/midi-synth/blob/master/src/midi_controller.rs
// https://crates.io/crates/nusb
// https://cdn.inmusicbrands.com/akai/attachments/APC%20mini%20mk2%20-%20Communication%20Protocol%20-%20v1.0.pdf

const APC_MINI_MK_2_VENDOR_ID: u16 = 0x0;
const APC_MINI_MK_2_PRODUCT_ID: u16 = 0x0;
const APC_MINI_MK_2_INTERFACE: u8 = 0x0;
const APC_MINI_MK_2_NAME: &str = "Akai APC Mini MK2";

pub struct ApcMiniMk2InputDeviceProfile {
    midi_device: Arc<Mutex<UsbMidiDevice>>,
    rx: mpsc::Receiver<MidiMessage>,
}

impl std::fmt::Debug for ApcMiniMk2InputDeviceProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApcMiniMk2InputDeviceProfile")
    }
}

impl ApcMiniMk2InputDeviceProfile {
    pub fn new() -> Result<Self, DemexInputDeviceError> {
        let (_, usb_device) = rusb::devices()
            .map_err(DemexInputDeviceError::RusbError)?
            .iter()
            .map(|device| (device.device_descriptor(), device))
            .find(|(device_descriptor, _)| {
                device_descriptor.as_ref().is_ok_and(|device_descriptor| {
                    device_descriptor.vendor_id() == APC_MINI_MK_2_VENDOR_ID
                        && device_descriptor.product_id() == APC_MINI_MK_2_PRODUCT_ID
                })
            })
            .ok_or(DemexInputDeviceError::InputDeviceNotFound(
                APC_MINI_MK_2_NAME.to_owned(),
            ))?;

        let handle = usb_device
            .open()
            .map_err(DemexInputDeviceError::RusbError)?;

        handle
            .claim_interface(APC_MINI_MK_2_INTERFACE)
            .map_err(DemexInputDeviceError::RusbError)?;

        let midi_device = Arc::new(Mutex::new(UsbMidiDevice::in_out_device(handle, 130, 1)));

        Ok(Self { midi_device, rx })
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
