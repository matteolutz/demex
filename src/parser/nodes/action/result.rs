use crate::{fixture::selection::FixtureSelection, input::event::DemexInputDeviceEvent};

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

    WithDeviceEvent {
        result: Box<ActionRunResult>,
        event: DemexInputDeviceEvent,
    },
}

impl ActionRunResult {
    pub fn new() -> Self {
        Self::Default
    }

    pub fn device_event(event: DemexInputDeviceEvent) -> Self {
        Self::WithDeviceEvent {
            result: Box::new(Self::new()),
            event,
        }
    }
}
