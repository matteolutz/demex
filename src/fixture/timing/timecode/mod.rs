use serde::{Deserialize, Serialize};
use state::TimecodeState;
use trigger::TimecodeTrigger;

pub mod state;
pub mod trigger;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timecode {
    id: u32,

    #[serde(default, skip_serializing, skip_deserializing)]
    state: TimecodeState,

    // Triggers are sorted by their frame
    triggers: Vec<TimecodeTrigger>,
}

impl Timecode {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn state(&self) -> &TimecodeState {
        &self.state
    }

    pub fn update(&mut self, new_frame: u64) {
        if let TimecodeState::Running {
            current_trigger_idx,
            current_frame,
            ..
        } = &mut self.state
        {
            loop {
                if *current_trigger_idx >= self.triggers.len()
                    || self.triggers[*current_trigger_idx].frame > new_frame
                {
                    break;
                }

                self.triggers[*current_trigger_idx].update(new_frame);
                *current_trigger_idx += 1;
            }

            *current_frame = new_frame;
        }
    }
}
