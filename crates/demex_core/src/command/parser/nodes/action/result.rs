use crate::{event::DemexEvent, patch::Patch, selection::FixtureSelection};

#[derive(Debug, Clone, Default)]
pub enum ActionRunResult {
    #[default]
    Default,

    Info(String),
    InfoWithLink(String, String),
    Warn(String),

    UpdateFixtureSelection(Option<FixtureSelection>),
    UpdatePatch(Patch),

    Lock,

    Save,

    WithEvents {
        result: Box<ActionRunResult>,
        events: Vec<DemexEvent>,
    },
}

impl ActionRunResult {
    pub fn new() -> Self {
        Self::Default
    }

    pub fn event(event: DemexEvent) -> Self {
        Self::WithEvents {
            result: Box::new(Self::new()),
            events: vec![event],
        }
    }

    pub fn events(events: Vec<DemexEvent>) -> Self {
        Self::WithEvents {
            result: Box::new(Self::new()),
            events,
        }
    }

    pub fn with_event(self, event: DemexEvent) -> Self {
        match self {
            Self::WithEvents { result, mut events } => {
                events.push(event);
                Self::WithEvents { result, events }
            }
            result => Self::WithEvents {
                result: Box::new(result),
                events: vec![event],
            },
        }
    }

    pub fn get_events(self) -> (Self, Option<Vec<DemexEvent>>) {
        match self {
            Self::WithEvents { result, events } => (*result, Some(events)),
            _ => (self, None),
        }
    }
}
