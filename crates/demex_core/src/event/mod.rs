use std::time;

use crate::{
    command::parser::nodes::object::Object, fixture::FixturePath, pool::PoolType,
    presets::preset::FixturePresetId,
};

mod selection;
pub use selection::*;

mod executor;
pub use executor::*;

pub mod list;

#[derive(Debug, Clone, PartialEq)]
pub enum DemexEvent {
    ExecutorGo(u32),
    ExecutorStop(u32),
    ExecutorFaderValueChanged {
        executor_id: u32,
        value: f32,
    },
    ExecutorUpdateEvent {
        id: u32,
        event: DemexExecutorUpdateEvent,
    },

    PoolItemAdded(PoolType, u32),
    PoolItemFlagsUpdated(PoolType, u32),
    PoolItemsDeleted {
        pool_type: PoolType,
        from_id: u32,
        to_id: u32,
    },
    PoolItemMoved {
        pool_type: PoolType,
        from_id: u32,
        to_id: u32,
    },

    GrandmasterFaderValueChanged(f32),

    GroupmasterValueChanged {
        group_master_id: u32,
        value: f32,
    },

    SpeedmasterFaderValueChanged {
        speed_master_id: u32,
        bpm: f32,
    },
    SpeedmasterTapped {
        speed_master_id: u32,
        instant: time::Instant,
    },

    GlobalEncoderValueChanged(u32),

    FixtureSelectionChanged(Option<FixtureSelectionWithGroup>),
    HighlightChanged(Option<FixtureSelectionWithGroup>),

    FixtureValuesChanged(Vec<FixturePath>),
    ObjectPropertyChanged(Object, String),

    KeyframeEffectUpdate(FixturePresetId),
}
