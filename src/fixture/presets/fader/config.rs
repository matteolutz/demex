use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DemexFaderConfig {
    Submaster { fixtures: Vec<u32> },
    SequenceRuntime { fixtures: Vec<u32>, runtime_id: u32 },
}

impl std::fmt::Display for DemexFaderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Submaster { fixtures: _ } => write!(f, "Sub"),
            Self::SequenceRuntime {
                fixtures: _,
                runtime_id,
            } => write!(f, "SR({})", runtime_id),
        }
    }
}
