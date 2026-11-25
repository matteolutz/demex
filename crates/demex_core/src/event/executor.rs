use std::time;

use crate::{event::DemexEvent, sequence::cue::CueIdx};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DemexExecutorUpdateEvent {
    CueActivate(CueIdx, time::Instant),
    CueDeactivate(CueIdx),
}

impl DemexExecutorUpdateEvent {
    pub fn into_event(self, id: u32) -> DemexEvent {
        DemexEvent::ExecutorUpdateEvent { id, event: self }
    }
}
