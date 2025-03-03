use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, EguiProbe, Serialize, Deserialize)]
pub enum DebugOutputVerbosity {
    Verbose,
    Quiet,

    #[default]
    Silent,
}
