use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub enum DemexProtoDeviceId {
    Controller,
    Node(u32),
}
