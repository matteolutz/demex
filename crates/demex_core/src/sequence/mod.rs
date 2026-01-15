use std::collections::HashSet;

use cue::{Cue, CueIdx};
use serde::{Deserialize, Serialize};

use crate::{fixture::FixturePath, implement_set_property, pool::PoolItem, sequence::cue::CueOut};

use super::{
    channel3::channel_value::FixtureChannelValue3, presets::PresetHandler,
    value_source::FixtureChannelValuePriority,
};

pub mod cue;
pub mod runtime;

pub mod frontend;

#[derive(Debug, Clone)]
pub struct FadeFixtureChannelValue {
    value: FixtureChannelValue3,
    alpha: f32,
    priority: FixtureChannelValuePriority,
}

impl FadeFixtureChannelValue {
    pub fn new(
        value: FixtureChannelValue3,
        alpha: f32,
        priority: FixtureChannelValuePriority,
    ) -> Self {
        Self {
            value,
            alpha,
            priority,
        }
    }

    pub fn home_ltp() -> FadeFixtureChannelValue {
        FadeFixtureChannelValue::new(
            FixtureChannelValue3::home(),
            1.0,
            FixtureChannelValuePriority::Ltp,
        )
    }

    pub fn home(priority: FixtureChannelValuePriority) -> FadeFixtureChannelValue {
        FadeFixtureChannelValue::new(FixtureChannelValue3::home(), 1.0, priority)
    }

    pub fn value(&self) -> &FixtureChannelValue3 {
        &self.value
    }

    pub fn into_value(self) -> FixtureChannelValue3 {
        self.value
    }

    pub fn flatten_value(self) -> Self {
        Self {
            value: self.value.flatten(),
            alpha: self.alpha,
            priority: self.priority,
        }
    }

    pub fn alpha(&self) -> f32 {
        self.alpha
    }

    pub fn priority(&self) -> FixtureChannelValuePriority {
        self.priority
    }

    pub fn set_alpha(&mut self, alpha: f32) {
        self.alpha = alpha;
    }

    pub fn multiply(mut self, fade: f32) -> Self {
        self.alpha *= fade;
        self
    }
}

#[derive(
    Debug,
    Copy,
    Clone,
    Serialize,
    Deserialize,
    Default,
    PartialEq,
    Eq,
    strum_macros::EnumString,
    strum_macros::Display,
)]
pub enum SequenceStopBehavior {
    #[default]
    ManualStop,

    Restart,

    AutoStop,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Sequence {
    pub(crate) id: u32,

    pub(crate) name: String,

    #[serde(default)]
    pub(crate) stop_behavior: SequenceStopBehavior,

    pub(crate) cues: Vec<Cue>,

    #[serde(default)]
    pub(crate) cue_out: CueOut,
}

impl Sequence {
    pub fn new(id: u32, name: String) -> Self {
        Self {
            id,
            name,
            cues: Vec::new(),
            stop_behavior: SequenceStopBehavior::default(),
            cue_out: CueOut::default(),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn stop_behavior(&self) -> SequenceStopBehavior {
        self.stop_behavior
    }

    pub fn stop_behavior_mut(&mut self) -> &mut SequenceStopBehavior {
        &mut self.stop_behavior
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    pub fn cue_out(&self) -> &CueOut {
        &self.cue_out
    }

    pub fn cue_out_mut(&mut self) -> &mut CueOut {
        &mut self.cue_out
    }

    pub fn add_cue(&mut self, cue: Cue) {
        self.cues.push(cue);
    }

    pub fn cues(&self) -> &Vec<Cue> {
        &self.cues
    }

    pub fn next_cue_idx(&self) -> CueIdx {
        if self.cues.is_empty() {
            (1, 0).into()
        } else {
            self.cues.last().unwrap().cue_idx().next_major()
        }
    }

    pub fn cues_mut(&mut self) -> &mut Vec<Cue> {
        &mut self.cues
    }

    pub fn cue(&self, idx: usize) -> &Cue {
        &self.cues[idx]
    }

    pub fn find_cue(&self, cue_idx: CueIdx) -> Option<&Cue> {
        self.cues.iter().find(|cue| cue.cue_idx() == cue_idx)
    }

    pub fn find_cue_mut(&mut self, cue_idx: CueIdx) -> Option<&mut Cue> {
        self.cues.iter_mut().find(|cue| cue.cue_idx() == cue_idx)
    }

    pub fn affected_fixtures(&self, preset_handler: &PresetHandler) -> HashSet<FixturePath> {
        self.cues
            .iter()
            .flat_map(|c| c.affected_fixtures(preset_handler))
            .collect()
    }
}

#[derive(strum_macros::EnumString, strum_macros::Display)]
pub enum SequenceProperty {
    Name,
    StopBehavior,

    CueOutFade,
    CueOutFadingFunction,
}

implement_set_property! {
    for Sequence with SequenceProperty,

    Name => name as String,
    StopBehavior => stop_behavior as SequenceStopBehavior,

    CueOutFade => cue_out.fade as f32,
    CueOutFadingFunction => cue_out.fading_function as cue::CueFadingFunction
}

impl From<&Sequence> for PoolItem {
    fn from(value: &Sequence) -> Self {
        PoolItem {
            id: value.id,
            name: value.name.clone().into(),
        }
    }
}
