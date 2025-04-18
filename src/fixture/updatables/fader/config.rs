use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::sequence::runtime::SequenceRuntime;

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, EguiProbe, Default)]
pub enum DemexFaderRuntimeFunction {
    #[default]
    Intensity,
    Speed,
    FadeAll,
}

#[derive(Debug, Serialize, Deserialize, Clone, EguiProbe)]
pub enum DemexFaderConfig {
    SequenceRuntime {
        runtime: SequenceRuntime,
        function: DemexFaderRuntimeFunction,
    },
}

impl Default for DemexFaderConfig {
    fn default() -> Self {
        todo!();
    }
}

impl std::fmt::Display for DemexFaderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SequenceRuntime {
                runtime: _,
                function: _,
            } => write!(f, "Seq"),
        }
    }
}
