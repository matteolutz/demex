use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub enum DebugOutputVerbosity {
    Verbose,
    Quiet,

    #[default]
    Silent,
}
