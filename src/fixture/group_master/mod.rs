use serde::{Deserialize, Serialize};

use crate::fixture::group_master::mode::GroupMasterMode;

pub mod mode;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupMaster {
    id: u32,

    group_id: u32,

    mode: GroupMasterMode,

    value: f32,
}

impl GroupMaster {
    pub fn new(id: u32, group_id: u32, mode: GroupMasterMode) -> Self {
        Self {
            id,
            group_id,
            mode,
            value: mode.default_value(),
        }
    }
}

impl GroupMaster {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn group_id(&self) -> u32 {
        self.group_id
    }

    pub fn mode(&self) -> GroupMasterMode {
        self.mode
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn value_mut(&mut self) -> &mut f32 {
        &mut self.value
    }

    pub fn apply(&self, fixture_value: f32) -> f32 {
        self.mode.apply(fixture_value, self.value)
    }
}
