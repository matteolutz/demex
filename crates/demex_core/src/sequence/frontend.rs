use itertools::Itertools;

use crate::{
    selection::FixtureSelection,
    sequence::{
        Sequence, SequenceStopBehavior,
        cue::{Cue, CueFadingFunction, CueIdx, CueOut, CueTiming, CueTrigger},
    },
};

#[derive(Debug, Clone)]
pub struct FrontendSequence {
    pub id: u32,

    pub name: String,

    pub stop_behavior: SequenceStopBehavior,

    pub cues: Vec<FrontendCue>,

    pub cue_out: CueOut,
}

impl From<&Sequence> for FrontendSequence {
    fn from(value: &Sequence) -> Self {
        Self {
            id: value.id,
            name: value.name.clone(),
            stop_behavior: value.stop_behavior,
            cues: value.cues.iter().map_into().collect(),
            cue_out: value.cue_out.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrontendCue {
    pub cue_idx: CueIdx,

    pub name: String,

    pub selection: FixtureSelection,

    // Time, to fade into the cue
    pub in_fade: f32,

    // Delay, before the cue starts fading in
    pub in_delay: f32,

    // When (as a percentage of the in_fade time), snapping of values, that are not
    // being faded, are changed.
    pub snap_percent: f32,

    pub block: bool,

    pub timing: CueTiming,

    pub trigger: CueTrigger,

    pub fading_function: CueFadingFunction,

    /// If the true, this cue will also move all channels except the intensity parameters of all fixtures
    /// in the next cue, that are not active in the current cue.
    pub move_in_black: bool,
}

impl From<&Cue> for FrontendCue {
    fn from(value: &Cue) -> Self {
        Self {
            cue_idx: value.cue_idx,
            name: value.name.clone(),
            selection: value.selection.clone(),
            in_fade: value.in_fade,
            in_delay: value.in_delay,
            snap_percent: value.snap_percent,
            block: value.block,
            timing: value.timing.clone(),
            trigger: value.trigger.clone(),
            fading_function: value.fading_function,
            move_in_black: value.move_in_black,
        }
    }
}
