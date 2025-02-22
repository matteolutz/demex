use crate::{fixture::selection::FixtureSelection, ui::window::edit::DemexEditWindow};

#[derive(Debug, Clone, Default)]
pub enum ActionRunResult {
    #[default]
    Default,

    Info(String),
    InfoWithLink(String, String),
    Warn(String),
    EditWindow(DemexEditWindow),

    UpdateSelectedFixtures(FixtureSelection),
}

impl ActionRunResult {
    pub fn new() -> Self {
        Self::Default
    }
}
