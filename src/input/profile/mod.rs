use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

pub mod akai;
pub mod generic;

#[derive(Debug, Serialize, Deserialize, EguiProbe, Clone)]
pub enum DemexInputDeviceProfileType {
    GenericMidi { midi_in_device: String },

    ApcMiniMk2 { apc_midi: String },
}

impl Default for DemexInputDeviceProfileType {
    fn default() -> Self {
        Self::GenericMidi {
            midi_in_device: "GenericMIDI".to_string(),
        }
    }
}
