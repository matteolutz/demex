use crate::ui::edit::DemexEditWindow;

#[derive(Debug, Clone)]
pub enum ActionRunResult {
    Default,
    Info(String),
    InfoWithLink(String, String),
    Warn(String),
    EditWindow(DemexEditWindow),
}

impl ActionRunResult {
    pub fn new() -> Self {
        Self::Default
    }
}
