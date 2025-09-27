use crate::{event::DemexEvent, selection::FixtureSelection};

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

    Lock,

    WithEvent {
        result: Box<ActionRunResult>,
        event: DemexEvent,
    },
}

impl ActionRunResult {
    pub fn new() -> Self {
        Self::Default
    }

    pub fn device_event(event: DemexEvent) -> Self {
        Self::WithEvent {
            result: Box::new(Self::new()),
            event,
        }
    }
}
