use crate::{
    event::{DemexEvent, FixtureSelectionWithGroup},
    patch::Patch,
};

#[derive(Debug, Clone, Default)]
pub enum ActionRunResult {
    #[default]
    Default,

    Info(String),
    InfoWithLink(String, String),
    Warn(String),

    UpdateFixtureSelection(Option<FixtureSelectionWithGroup>),
    UpdateHighlight(Option<FixtureSelectionWithGroup>),
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

    pub fn with_optional_event(self, event: Option<DemexEvent>) -> Self {
        let Some(event) = event else {
            return self;
        };

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

    pub fn with_event(self, event: DemexEvent) -> Self {
        self.with_optional_event(Some(event))
    }

    pub fn with_events(self, new_events: impl IntoIterator<Item = DemexEvent>) -> Self {
        match self {
            Self::WithEvents { result, mut events } => {
                events.extend(new_events);
                Self::WithEvents { result, events }
            }
            result => Self::WithEvents {
                result: Box::new(result),
                events: new_events.into_iter().collect(),
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
