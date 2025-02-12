use crate::{parser::nodes::fixture_selector::FixtureSelector, ui::window::edit::DemexEditWindow};

#[derive(Debug, Clone, Default)]
pub enum ActionRunResult {
    #[default]
    Default,

    Info(String),
    InfoWithLink(String, String),
    Warn(String),
    EditWindow(DemexEditWindow),

    UpdateSelectedFixtures(FixtureSelector),
}

impl ActionRunResult {
    pub fn new() -> Self {
        Self::Default
    }
}
