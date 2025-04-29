use serde::{Deserialize, Serialize};

use crate::fixture::{
    handler::FixtureHandler,
    presets::{preset::FixturePresetId, PresetHandler},
    selection::FixtureSelection,
    updatables::UpdatableHandler,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimecodeTriggerType {
    SetPreset {
        selection: FixtureSelection,
        preset: FixturePresetId,
    },
    ExecutorStartAndNext(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimecodeTrigger {
    pub trigger_type: TimecodeTriggerType,
    pub millis: u64,
}

impl TimecodeTrigger {
    pub fn update(
        &mut self,
        new_millis: u64,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
        updatable_handler: &mut UpdatableHandler,
    ) {
        let time_offset = (new_millis - self.millis) as f32 / 1000.0;

        match self.trigger_type {
            TimecodeTriggerType::ExecutorStartAndNext(executor_id) => {
                let _ = updatable_handler.start_or_next_executor(
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
