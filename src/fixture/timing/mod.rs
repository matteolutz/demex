use std::{collections::HashMap, time};

use error::TimingHandlerError;
use serde::{Deserialize, Serialize};
use speed_master::SpeedMasterValue;
use timecode::Timecode;

use crate::input::timecode::packet::TimecodePacket;

pub mod error;
pub mod speed_master;
pub mod tap;
pub mod timecode;

#[derive(Serialize, Deserialize, Clone)]
pub struct TimingHandler {
    speed_master_values: HashMap<u32, SpeedMasterValue>,

    timecodes: HashMap<u32, Timecode>,
}

impl Default for TimingHandler {
    fn default() -> Self {
        Self {
            speed_master_values: HashMap::from_iter(
                (0u32..10u32).map(|id| (id, SpeedMasterValue::default())),
            ),
            timecodes: HashMap::new(),
        }
    }
}

impl TimingHandler {
    pub fn speed_master_values(&self) -> &HashMap<u32, SpeedMasterValue> {
        &self.speed_master_values
    }

    pub fn speed_master_values_mut(&mut self) -> &mut HashMap<u32, SpeedMasterValue> {
        &mut self.speed_master_values
    }

    pub fn get_speed_master_value(&self, id: u32) -> Result<&SpeedMasterValue, TimingHandlerError> {
        self.speed_master_values
            .get(&id)
            .ok_or(TimingHandlerError::SpeedMasterValueNotFound(id))
    }

    pub fn get_speed_master_value_mut(
        &mut self,
        id: u32,
    ) -> Result<&mut SpeedMasterValue, TimingHandlerError> {
        self.speed_master_values
            .get_mut(&id)
            .ok_or(TimingHandlerError::SpeedMasterValueNotFound(id))
    }

    pub fn tap_speed_master_value(
        &mut self,
        id: u32,
        interval: time::Instant,
    ) -> Result<(), TimingHandlerError> {
        let speed_master_value = self.get_speed_master_value_mut(id)?;
        speed_master_value.tap(interval);
        Ok(())
    }
}

impl TimingHandler {
    pub fn timecodes(&self) -> &HashMap<u32, Timecode> {
        &self.timecodes
    }

    pub fn timecodes_mut(&mut self) -> &mut HashMap<u32, Timecode> {
        &mut self.timecodes
    }

    pub fn get_timecode(&self, id: u32) -> Result<&Timecode, TimingHandlerError> {
        self.timecodes
            .get(&id)
            .ok_or(TimingHandlerError::TimecodeNotFound(id))
    }

    pub fn update_running_timecodes(&mut self, timecode_packet: TimecodePacket) {
        let new_frame = timecode_packet.frame();

        self.timecodes
            .iter_mut()
            .filter(|(_, timecode)| timecode.state().is_running())
            .for_each(|(_, timecode)| timecode.update(new_frame));
    }
}
