use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

pub mod akai;

#[derive(Debug, Serialize, Deserialize, EguiProbe, Default)]
pub enum DemexInputDeviceProfileType {
    #[default]
    ApcMiniMk2,
}
