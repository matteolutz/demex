use serde::{Deserialize, Serialize};

use crate::fixture::{presets::preset::FixturePresetId, selection::FixtureSelection};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimecodeTriggerType {
    SetPreset {
        selection: FixtureSelection,
        preset: FixturePresetId,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimecodeTrigger {
    pub trigger_type: TimecodeTriggerType,
    pub frame: u64,
}

impl TimecodeTrigger {
    pub fn update(&mut self, _new_frame: u64) {
        log::info!("updating")
    }
}
