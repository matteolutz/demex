use serde::{Deserialize, Serialize};

pub mod akai;
pub mod behringer;
pub mod debug;
pub mod midi_timecode;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DemexInputDeviceProfileType {
    MidiTimecode { midi_in_device: String },

    ApcMiniMk2 { apc_midi: String },

    BehringerXTouchCompact { xtouch_midi: String },

    Debug,
}

impl Default for DemexInputDeviceProfileType {
    fn default() -> Self {
        Self::MidiTimecode {
            midi_in_device: "MidiTimecode".to_string(),
        }
    }
}
