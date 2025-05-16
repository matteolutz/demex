use std::{collections::HashMap, time};

use error::TimingHandlerError;
use serde::{Deserialize, Serialize};
use speed_master::SpeedMasterValue;
use timecode::Timecode;

use crate::input::{midi::MidiQuarterTimecodePiece, timecode::packet::TimecodePacket};

use super::{handler::FixtureHandler, presets::PresetHandler, updatables::UpdatableHandler};

pub mod error;
pub mod speed_master;
pub mod tap;
pub mod timecode;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimingHandler {
    speed_master_values: HashMap<u32, SpeedMasterValue>,

    timecodes: HashMap<u32, Timecode>,

    #[serde(default, skip_serializing, skip_deserializing)]
    current_timecode_packet: TimecodePacket,
}

impl Default for TimingHandler {
    fn default() -> Self {
        Self {
            speed_master_values: HashMap::from_iter(
                (0u32..10u32).map(|id| (id, SpeedMasterValue::default())),
            ),
            timecodes: HashMap::new(),
            current_timecode_packet: TimecodePacket::default(),
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

    fn update_running_timecodes(
        &mut self,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
        updatable_handler: &mut UpdatableHandler,
    ) {
        self.timecodes.values_mut().for_each(|timecode| {
            timecode.update(
                self.current_timecode_packet.millis(),
                fixture_handler,
                preset_handler,
                updatable_handler,
            )
        });
    }

    pub fn update_timecode(
        &mut self,
        timecode_packet: TimecodePacket,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
        updatable_handler: &mut UpdatableHandler,
    ) {
        self.current_timecode_packet = timecode_packet;

        self.update_running_timecodes(fixture_handler, preset_handler, updatable_handler);
    }

    pub fn update_timecode_quarter_frame(
        &mut self,
        piece: MidiQuarterTimecodePiece,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
        updatable_handler: &mut UpdatableHandler,
    ) {
        self.current_timecode_packet.update_from(piece);

        self.update_running_timecodes(fixture_handler, preset_handler, updatable_handler);
    }

    pub fn current_timecode_packet(&self) -> &TimecodePacket {
        &self.current_timecode_packet
    }
}
