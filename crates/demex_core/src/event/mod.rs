use crate::{command::parser::nodes::object::Object, selection::FixtureSelection};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DemexEvent {
    ExecutorGo(u32),
    ExecutorStop(u32),
    ExecutorFaderValueChanged(u32),

    GrandmasterFaderValueChanged,

    GroupmasterValueChanged(u32),

    SpeedmasterFaderValueChanged(u32),

    GlobalEncoderValueChanged(u32),

    FixtureSelectionChanged(Option<FixtureSelection>),

    FixtureValuesChanged(Vec<u32>),
    ObjectPropertyChanged(Object, String),
}
