use crate::command::parser::nodes::{fixture_selector::FixtureSelector, object::Object};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DemexEvent {
    ExecutorGo(u32),
    ExecutorStop(u32),
    ExecutorFaderValueChanged(u32),

    GrandmasterFaderValueChanged,

    GroupmasterValueChanged(u32),

    SpeedmasterFaderValueChanged(u32),

    GlobalEncoderValueChanged(u32),

    FixtureSelectionChanged(FixtureSelector),

    ObjectPropertyChanged(Object, String),
}
