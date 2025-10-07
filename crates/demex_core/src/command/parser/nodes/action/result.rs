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

    UpdateFixtureSelection(Option<FixtureSelection>),

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

    pub fn get_event(self) -> (Self, Option<DemexEvent>) {
        match self {
            Self::WithEvent { result, event } => (*result, Some(event)),
            _ => (self, None),
        }
    }
}
