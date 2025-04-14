use std::collections::HashSet;

use cue::{Cue, CueIdx};
use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use super::{
    channel2::channel_value::FixtureChannelValue2, presets::PresetHandler,
    value_source::FixtureChannelValuePriority,
};

pub mod cue;
pub mod runtime;

#[derive(Debug, Clone)]
pub struct FadeFixtureChannelValue {
    value: FixtureChannelValue2,
    alpha: f32,
    priority: FixtureChannelValuePriority,
}

impl FadeFixtureChannelValue {
    pub fn new(
        value: FixtureChannelValue2,
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
            FixtureChannelValue2::Home,
            1.0,
            FixtureChannelValuePriority::Ltp,
        )
    }

    pub fn value(&self) -> &FixtureChannelValue2 {
        &self.value
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

    pub fn multiply(mut self, fade: f32) -> Self {
        self.alpha *= fade;
        self
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Default, PartialEq, Eq, EguiProbe)]
pub enum SequenceStopBehavior {
    #[default]
    ManualStop,

    Restart,

    AutoStop,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, EguiProbe)]
pub struct Sequence {
    #[egui_probe(skip)]
    id: u32,

    name: String,

    #[serde(default)]
    stop_behavior: SequenceStopBehavior,

    #[egui_probe(skip)]
    cues: Vec<Cue>,
}

impl Sequence {
    pub fn new(id: u32, name: String) -> Self {
        Self {
            id,
            name,
            cues: Vec::new(),
            stop_behavior: SequenceStopBehavior::default(),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn stop_behavior(&self) -> SequenceStopBehavior {
        self.stop_behavior
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    pub fn add_cue(&mut self, cue: Cue) {
        self.cues.push(cue);
    }

    pub fn cues(&self) -> &Vec<Cue> {
        &self.cues
    }

    pub fn next_cue_idx(&self) -> CueIdx {
        if self.cues.is_empty() {
            (1, 0)
        } else {
            (self.cues.last().unwrap().cue_idx().0 + 1, 0)
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

    pub fn affected_fixtures(&self, preset_handler: &PresetHandler) -> HashSet<u32> {
        self.cues
            .iter()
            .flat_map(|c| c.affected_fixtures(preset_handler))
            .collect()
    }
}
