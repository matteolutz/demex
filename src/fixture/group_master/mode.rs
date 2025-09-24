use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupMasterMode {
    Positive,
}

impl GroupMasterMode {
    pub fn default_value(&self) -> f32 {
        match self {
            Self::Positive => 1.0,
        }
    }

    pub fn apply(&self, fixture_value: f32, master_value: f32) -> f32 {
        match self {
            Self::Positive => fixture_value * master_value,
        }
    }
}
