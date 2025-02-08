use serde::{Deserialize, Serialize};

use crate::fixture::sequence::cue::CueIdx;

use super::{action::Action, fixture_selector::FixtureSelector};

pub trait ObjectTrait {
    fn default_action(self) -> Option<Action>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HomeableObject {
    FixtureSelector(FixtureSelector),
    Executor(u32),
    Fader(u32),
}

impl ObjectTrait for HomeableObject {
    fn default_action(self) -> Option<Action> {
        match self {
            Self::FixtureSelector(fixture_selector) => {
                Some(Action::FixtureSelector(fixture_selector))
            }
            Self::Executor(executor_id) => Some(Action::EditExecutor(executor_id)),
            Self::Fader(fader_id) => Some(Action::EditFader(fader_id)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Object {
    HomeableObject(HomeableObject),
    Sequence(u32),
    SequenceCue(u32, CueIdx),
    Preset(u16, u32),
}

impl ObjectTrait for Object {
    fn default_action(self) -> Option<Action> {
        match self {
            Self::HomeableObject(homeable_object) => homeable_object.default_action(),
            Self::Sequence(sequence_id) => Some(Action::EditSequence(sequence_id)),
            Self::SequenceCue(sequence_id, cue_idx) => {
                Some(Action::EditSequenceCue(sequence_id, cue_idx))
            }
            Self::Preset(preset_id, fixture_id) => Some(Action::EditPreset(preset_id, fixture_id)),
        }
    }
}
