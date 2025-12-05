use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DebugOutputVerbosity {
    Verbose,
    Quiet,

    #[default]
    Silent,
}
