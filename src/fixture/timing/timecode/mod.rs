use serde::{Deserialize, Serialize};
use state::TimecodeState;
use trigger::TimecodeTrigger;

use crate::fixture::{
    handler::FixtureHandler, presets::PresetHandler, updatables::UpdatableHandler,
};

pub mod state;
pub mod trigger;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timecode {
    id: u32,

    name: String,

    #[serde(default, skip_serializing, skip_deserializing)]
    state: TimecodeState,

    // Triggers are sorted by their frame
    triggers: Vec<TimecodeTrigger>,
}

impl Timecode {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn state(&self) -> &TimecodeState {
        &self.state
    }

    pub fn update(
        &mut self,
        new_millis: u64,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
        updatable_handler: &mut UpdatableHandler,
    ) {
        if let TimecodeState::Running {
            current_trigger_idx,
            current_millis,
            ..
        } = &mut self.state
        {
            loop {
                if *current_trigger_idx >= self.triggers.len()
                    || self.triggers[*current_trigger_idx].millis > new_millis
                {
                    break;
                }

                self.triggers[*current_trigger_idx].update(
                    new_millis,
                    fixture_handler,
                    preset_handler,
                    updatable_handler,
                );
                *current_trigger_idx += 1;
            }

            *current_millis = new_millis;
        }
    }
}
