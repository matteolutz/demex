use serde::{Deserialize, Serialize};

pub mod akai;
pub mod midi_timecode;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub enum DemexInputDeviceProfileType {
    MidiTimecode { midi_in_device: String },

    ApcMiniMk2 { apc_midi: String },
}

impl Default for DemexInputDeviceProfileType {
    fn default() -> Self {
        Self::MidiTimecode {
            midi_in_device: "MidiTimecode".to_string(),
        }
    }
}
