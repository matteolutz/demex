use crate::{command::parser::nodes::object::Object, pool::PoolType};

mod selection;
pub use selection::*;

mod executor;
pub use executor::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DemexEvent {
    ExecutorGo(u32),
    ExecutorStop(u32),
    ExecutorFaderValueChanged(u32),
    ExecutorUpdateEvent {
        id: u32,
        event: DemexExecutorUpdateEvent,
    },

    PoolItemAdded(PoolType, u32),
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

    GrandmasterFaderValueChanged,

    GroupmasterValueChanged(u32),

    SpeedmasterFaderValueChanged(u32),

    GlobalEncoderValueChanged(u32),

    FixtureSelectionChanged(Option<FixtureSelectionWithGroup>),

    FixtureValuesChanged(Vec<u32>),
    ObjectPropertyChanged(Object, String),
}
