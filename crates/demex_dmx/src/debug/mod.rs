use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Serialize, Deserialize, strum::EnumIter)]
pub enum DebugOutputVerbosity {
    Verbose,
    Quiet,

    #[default]
    Silent,
}
