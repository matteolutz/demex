use std::time;

use crate::sequence::cue::CueIdx;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DemexExecutorUpdateEvent {
    CueActivate(CueIdx, time::Instant),
    CueDeactivate(CueIdx),
}
