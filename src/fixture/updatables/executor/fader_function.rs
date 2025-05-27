use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub enum DemexExecutorFaderFunction {
    #[default]
    Intensity,
    Speed,
    FadeAll,
}
