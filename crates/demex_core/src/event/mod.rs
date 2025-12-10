use crate::{command::parser::nodes::object::Object, pool::PoolType, selection::FixtureSelection};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DemexEvent {
    ExecutorGo(u32),
    ExecutorStop(u32),
    ExecutorFaderValueChanged(u32),

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

    FixtureSelectionChanged(Option<FixtureSelection>),

    FixtureValuesChanged(Vec<u32>),
    ObjectPropertyChanged(Object, String),
}
