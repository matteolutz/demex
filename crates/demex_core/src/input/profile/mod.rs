use serde::{Deserialize, Serialize};

use crate::input::{error::DemexInputDeviceError, midi::device::MidiInOutIdentifier};

pub mod akai;
pub mod behringer;
pub mod debug;
pub mod midi_timecode;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum DemexInputDeviceProfileType {
    MidiTimecode {
        midi_in_device: String,
    },

    ApcMiniMk2 {
        #[serde(default)]
        midi_id: MidiInOutIdentifier,
    },

    BehringerXTouchCompact {
        #[serde(default)]
        midi_id: MidiInOutIdentifier,
    },

    Debug,
}

impl Default for DemexInputDeviceProfileType {
    fn default() -> Self {
        Self::MidiTimecode {
            midi_in_device: "MidiTimecode".to_string(),
        }
    }
}

impl DemexInputDeviceProfileType {
    pub fn get_connected_devices(&self) -> Result<Vec<Self>, DemexInputDeviceError> {
        Ok(vec![])
    }
}
