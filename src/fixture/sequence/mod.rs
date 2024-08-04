use cue::Cue;

pub mod cue;
pub mod runtime;

#[derive(Debug, Clone)]
pub struct Sequence {
    id: u32,
    cues: Vec<Cue>,
}

impl Sequence {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            cues: Vec::new(),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
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
