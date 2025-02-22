use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::{selection::FixtureSelection, sequence::runtime::SequenceRuntime};

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, EguiProbe, Default)]
pub enum DemexFaderRuntimeFunction {
    #[default]
    Intensity,
    Speed,
}

#[derive(Debug, Serialize, Deserialize, Clone, EguiProbe)]
pub enum DemexFaderConfig {
    Submaster {
        selection: FixtureSelection,
    },
    SequenceRuntime {
        selection: FixtureSelection,
        runtime: SequenceRuntime,
        function: DemexFaderRuntimeFunction,
    },
}

impl Default for DemexFaderConfig {
    fn default() -> Self {
        Self::Submaster {
            selection: Vec::default().into(),
        }
    }
}

impl std::fmt::Display for DemexFaderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Submaster { selection: _ } => write!(f, "Sub"),
            Self::SequenceRuntime {
                selection: _,
                runtime: _,
                function: _,
            } => write!(f, "Seq"),
        }
    }
}
