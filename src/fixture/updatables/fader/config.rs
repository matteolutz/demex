use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub enum DemexFaderRuntimeFunction {
    #[default]
    Intensity,
    Speed,
    FadeAll,
}
