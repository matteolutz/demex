use cue::Cue;

use super::channel::value::FixtureChannelValue;

pub mod cue;
pub mod runtime;

#[derive(Debug, Clone)]
pub struct FadeFixtureChannelValue {
    value: FixtureChannelValue,
    alpha: f32,
}

impl FadeFixtureChannelValue {
    pub fn new(value: FixtureChannelValue, alpha: f32) -> Self {
        Self { value, alpha }
    }

    pub fn value(&self) -> &FixtureChannelValue {
        &self.value
    }

    pub fn alpha(&self) -> f32 {
        self.alpha
    }
}

#[derive(Debug, Clone)]
pub struct Sequence {
    cues: Vec<Cue>,
}

impl Sequence {
    pub fn new() -> Self {
        Self { cues: Vec::new() }
    }

    pub fn add_cue(&mut self, cue: Cue) {
        self.cues.push(cue);
    }

    pub fn cues(&self) -> &Vec<Cue> {
        &self.cues
    }

    pub fn cue(&self, idx: usize) -> &Cue {
        &self.cues[idx]
    }
}
