use std::collections::HashSet;

use demex_core::{channel3::feature::feature_group::FixtureChannel3FeatureGroup, pool::PoolType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MultiPoolConfig {
    pub(super) pools: Vec<MultiPoolEntry>,

    pub(super) size: (u16, u16),
}

impl MultiPoolConfig {
    pub fn example() -> Self {
        Self {
            size: (20, 10),

            pools: vec![
                MultiPoolEntry {
                    pool_type: PoolType::Group,
                    start_cell: (0, 0),
                    size: (5, 2),
                    start_id: 0,
                },
                MultiPoolEntry {
                    pool_type: PoolType::Preset(FixtureChannel3FeatureGroup::Dimmer),
                    start_cell: (5, 0),
                    size: (5, 2),
                    start_id: 0,
                },
                MultiPoolEntry {
                    pool_type: PoolType::Executor,
                    start_cell: (0, 2),
                    size: (5, 2),
                    start_id: 0,
                },
            ],
        }
    }

    pub fn pool_types(&self) -> HashSet<PoolType> {
        self.pools.iter().map(|p| p.pool_type).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiPoolEntry {
    pub(super) pool_type: PoolType,
    pub(super) start_cell: (u16, u16),
    pub(super) size: (u16, u16),

    pub(super) start_id: u32,
}

impl MultiPoolEntry {
    pub(super) fn num_items(&self) -> u32 {
        (self.size.0 as u32 * self.size.1 as u32) - 1
    }

    pub(super) fn get_cell(&self, idx: usize) -> (u16, u16) {
        let idx = idx + 1; // account for entry header

        let (width, _) = self.size;
        let row_offset = idx / width as usize;
        let col_offset = idx % width as usize;

        (col_offset as u16, row_offset as u16)
    }

    pub(super) fn resize(&mut self, (mouse_col, mouse_row): (u16, u16)) {
        self.size.0 = (mouse_col as i16 - self.start_cell.0 as i16).max(1) as u16;
        self.size.1 = (mouse_row as i16 - self.start_cell.1 as i16).max(1) as u16;
    }
}
