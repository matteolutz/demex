use crate::fixture::selection::FixtureSelection;

#[derive(Debug, Clone, Default)]
pub enum ActionRunResult {
    #[default]
    Default,

    Info(String),
    InfoWithLink(String, String),
    Warn(String),

    #[cfg(feature = "ui")]
    EditWindow(crate::ui::window::edit::DemexEditWindow),

    UpdateSelectedFixtures(Option<FixtureSelection>),
}

impl ActionRunResult {
    pub fn new() -> Self {
        Self::Default
    }
}
