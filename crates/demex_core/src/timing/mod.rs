use std::{collections::HashMap, time};

use error::TimingHandlerError;
use serde::{Deserialize, Serialize};
use speed_master::SpeedMasterValue;
use timecode::Timecode;

use crate::{
    engine::component::Component,
    event::{DemexEvent, list::DemexEventList},
    input::{
        midi::MidiQuarterTimecodePiece,
        timecode::{packet::TimecodePacket, synchronizer::TimecodeSynchronizer},
    },
    state::fixture_state_handler::FixtureStateHandler,
};

use super::{presets::PresetHandler, updatables::UpdatableHandler};

pub mod error;
pub mod speed_master;
pub mod tap;
pub mod timecode;

impl Component for TimingHandler {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimingHandler {
    speed_master_values: HashMap<u32, SpeedMasterValue>,

    timecodes: HashMap<u32, Timecode>,

    #[serde(default, skip_serializing, skip_deserializing)]
    timecode_synchronizer: TimecodeSynchronizer,
}

impl Default for TimingHandler {
    fn default() -> Self {
        Self {
            speed_master_values: HashMap::from_iter(
                (0u32..10u32).map(|id| (id, SpeedMasterValue::default())),
            ),
            timecodes: HashMap::new(),
            timecode_synchronizer: TimecodeSynchronizer::default(),
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
        event_list: &mut DemexEventList,
    ) -> Result<(), TimingHandlerError> {
        let speed_master_value = self.get_speed_master_value_mut(id)?;

        if let Some(new_bpm) = speed_master_value.tap(interval) {
            event_list.push(DemexEvent::SpeedmasterFaderValueChanged {
                speed_master_id: id,
                bpm: new_bpm,
            });
        }

        event_list.push(DemexEvent::SpeedmasterTapped {
            speed_master_id: id,
            instant: interval,
        });

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

    pub fn timecode_synchronizer(&self) -> &TimecodeSynchronizer {
        &self.timecode_synchronizer
    }

    pub fn update_running_timecodes(
        &mut self,
        fixture_handler: &mut FixtureStateHandler,
        preset_handler: &PresetHandler,
        updatable_handler: &mut UpdatableHandler,
        event_list: &mut DemexEventList,
    ) {
        self.timecode_synchronizer.update_estimated();

        self.timecodes.values_mut().for_each(|timecode| {
            timecode.update(
                self.timecode_synchronizer.estimated_millis(),
                fixture_handler,
                preset_handler,
                updatable_handler,
                event_list,
            )
        });
    }

    fn recalculate_timecode_indices(&mut self) {
        let current_millis = self.timecode_synchronizer.estimated_millis();

        self.timecodes.values_mut().for_each(|timecode| {
            timecode
                .scheduler_mut()
                .recalculate_next_trigger(current_millis);
        });
    }

    pub fn handle_timecode_packet(&mut self, timecode_packet: TimecodePacket) {
        if self
            .timecode_synchronizer
            .process_new_timecode(timecode_packet)
        {
            self.recalculate_timecode_indices();
        }
    }

    pub fn handle_timecode_quarter_frame(&mut self, piece: MidiQuarterTimecodePiece) {
        if self.timecode_synchronizer.process_new_quarter_frame(piece) {
            self.recalculate_timecode_indices();
        }
    }
}
