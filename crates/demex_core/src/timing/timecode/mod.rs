use serde::{Deserialize, Serialize};
use state::TimecodeState;
use trigger::TimecodeTrigger;

use crate::{
    event::list::DemexEventList, presets::PresetHandler,
    state::fixture_state_handler::FixtureStateHandler,
    timing::timecode::scheduler::TimecodeTriggerScheduler, updatables::UpdatableHandler,
};

pub mod scheduler;
pub mod state;
pub mod synchronizer;
pub mod trigger;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableTimecode {
    id: u32,
    timecode_slot: u32,
    name: String,
    triggers: Vec<TimecodeTrigger>,
}

impl From<&Timecode> for SerializableTimecode {
    fn from(value: &Timecode) -> Self {
        Self {
            id: value.id,
            timecode_slot: value.timecode_slot,
            name: value.name.clone(),
            triggers: value.scheduler.triggers().to_vec(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Timecode {
    id: u32,

    timecode_slot: u32,

    name: String,

    state: TimecodeState,

    scheduler: TimecodeTriggerScheduler,
}

impl Serialize for Timecode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerializableTimecode::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Timecode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        SerializableTimecode::deserialize(deserializer).map(|serializable_timecode| Self {
            id: serializable_timecode.id,
            timecode_slot: serializable_timecode.timecode_slot,
            name: serializable_timecode.name,
            state: TimecodeState::default(),
            scheduler: TimecodeTriggerScheduler::new(serializable_timecode.triggers),
        })
    }
}

impl Timecode {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn timecode_slot(&self) -> u32 {
        self.timecode_slot
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn state(&self) -> &TimecodeState {
        &self.state
    }

    pub fn scheduler(&self) -> &TimecodeTriggerScheduler {
        &self.scheduler
    }

    pub fn scheduler_mut(&mut self) -> &mut TimecodeTriggerScheduler {
        &mut self.scheduler
    }

    pub fn add_trigger(&mut self, trigger: TimecodeTrigger) {
        self.scheduler.add_trigger(trigger);
    }

    pub fn update(
        &mut self,
        new_millis: u64,
        fixture_handler: &mut FixtureStateHandler,
        preset_handler: &PresetHandler,
        updatable_handler: &mut UpdatableHandler,
        event_list: &mut DemexEventList,
    ) {
        if !self.state.is_running() {
            return;
        }

        for trigger in self.scheduler.update(new_millis) {
            trigger.trigger(
                new_millis,
                fixture_handler,
                preset_handler,
                updatable_handler,
                event_list,
            );
        }
    }
}
