use serde::{Deserialize, Serialize};

use crate::{
    fixture::handler::FixtureHandler,
    presets::{PresetHandler, preset::FixturePresetId},
    selection::FixtureSelection,
    updatables::UpdatableHandler,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimecodeTriggerType {
    SetPreset {
        selection: FixtureSelection,
        preset: FixturePresetId,
    },
    ExecutorGo(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimecodeTrigger {
    pub trigger_type: TimecodeTriggerType,
    pub millis: u64,
}

impl PartialOrd for TimecodeTrigger {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.millis.partial_cmp(&other.millis)
    }
}

impl Ord for TimecodeTrigger {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.millis.cmp(&other.millis)
    }
}

impl TimecodeTrigger {
    pub fn trigger(
        &self,
        trigger_millis: u64,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
        updatable_handler: &mut UpdatableHandler,
    ) {
        let time_offset = (trigger_millis - self.millis) as f32 / 1000.0;

        match self.trigger_type {
            TimecodeTriggerType::ExecutorGo(executor_id) => {
                let _ = updatable_handler.executor_go(
                    executor_id,
                    fixture_handler,
                    preset_handler,
                    time_offset,
                );
            }
            _ => todo!(),
        }
    }
}
