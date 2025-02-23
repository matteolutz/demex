use std::{collections::HashMap, time};

use error::TimingHandlerError;
use serde::{Deserialize, Serialize};
use speed_master::SpeedMasterValue;

pub mod error;
pub mod speed_master;
pub mod tap;

#[derive(Serialize, Deserialize, Clone)]
pub struct TimingHandler {
    speed_master_values: HashMap<u32, SpeedMasterValue>,
}

impl Default for TimingHandler {
    fn default() -> Self {
        Self {
            speed_master_values: HashMap::from_iter(
                (0u32..10u32).map(|id| (id, SpeedMasterValue::default())),
            ),
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
