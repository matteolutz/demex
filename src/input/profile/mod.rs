use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

pub mod akai;
pub mod generic;

#[derive(Debug, Serialize, Deserialize, EguiProbe, Default, Copy, Clone)]
pub enum DemexInputDeviceProfileType {
    #[default]
    GenericMidi,

    ApcMiniMk2,
}
