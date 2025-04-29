use std::collections::HashMap;

use crate::fixture::channel3::channel_value::FixtureChannelValue3;

#[derive(Debug, Clone, Default)]
pub enum TimecodeState {
    #[default]
    Stopped,

    Running {
        current_trigger_idx: usize,
        current_millis: u64,
        timecode_values: HashMap<u32, HashMap<String, FixtureChannelValue3>>,
    },
}

impl TimecodeState {
    pub fn is_running(&self) -> bool {
        matches!(self, TimecodeState::Running { .. })
    }
}
