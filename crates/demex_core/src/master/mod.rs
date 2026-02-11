use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct MasterConfig {
    group_masters: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct MasterHandler {
    grandmaster: f32,

    group_master_value: HashMap<u32, f32>,
}

impl From<&MasterHandler> for MasterConfig {
    fn from(value: &MasterHandler) -> Self {
        MasterConfig {
            group_masters: value.group_master_value.keys().copied().collect(),
        }
    }
}

impl MasterHandler {
    pub fn new(config: MasterConfig) -> Self {
        Self {
            grandmaster: 1.0,
            group_master_value: config
                .group_masters
                .into_iter()
                .map(|group_id| (group_id, 1.0))
                .collect(),
        }
    }
}
