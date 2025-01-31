use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::sequence::runtime::SequenceRuntime;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, EguiProbe, Default)]
pub enum DemexFaderRuntimeFunction {
    #[default]
    Speed,
    Intensity,
}

#[derive(Debug, Serialize, Deserialize, Clone, EguiProbe)]
pub enum DemexFaderConfig {
    Submaster {
        fixtures: Vec<u32>,
    },
    SequenceRuntime {
        fixtures: Vec<u32>,
        runtime: SequenceRuntime,
        function: DemexFaderRuntimeFunction,
    },
}

impl Default for DemexFaderConfig {
    fn default() -> Self {
        Self::Submaster {
            fixtures: Vec::default(),
        }
    }
}

impl std::fmt::Display for DemexFaderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Submaster { fixtures: _ } => write!(f, "Sub"),
            Self::SequenceRuntime {
                fixtures: _,
                runtime: _,
                function: _,
            } => write!(f, "Seq"),
        }
    }
}
