use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

pub mod akai;
pub mod midi_timecode;

#[derive(Debug, Serialize, Deserialize, EguiProbe, Clone)]
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
