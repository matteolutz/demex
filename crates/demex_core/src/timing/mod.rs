use std::{collections::HashMap, time};

use demex_dmx::timecode::TimedTimecodePacket;
use error::TimingHandlerError;
use serde::{Deserialize, Serialize};
use speed_master::SpeedMasterValue;
use timecode::Timecode;

use crate::{
    engine::component::Component,
    event::{DemexEvent, list::DemexEventList},
    input::midi::MidiQuarterTimecodePiece,
    state::fixture_state_handler::FixtureStateHandler,
    timing::timecode::synchronizer::TimecodeSynchronizer,
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
    timecode_slots: HashMap<u32, TimecodeSynchronizer>,
}

impl Default for TimingHandler {
    fn default() -> Self {
        Self {
            speed_master_values: HashMap::from_iter(
                (0u32..10u32).map(|id| (id, SpeedMasterValue::default())),
            ),
            timecodes: HashMap::new(),
            timecode_slots: HashMap::new(),
        }
    }
}

impl TimingHandler {
    pub fn update_speedmasters(&mut self) {
        for speedmaster in self.speed_master_values.values_mut() {
            speedmaster.update();
        }
    }

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

    pub fn timecode_synchronizer(&mut self, id: u32) -> &TimecodeSynchronizer {
        self.timecode_slots.entry(id).or_default()
    }

    pub fn update_running_timecodes(
        &mut self,
        fixture_handler: &mut FixtureStateHandler,
        preset_handler: &PresetHandler,
        updatable_handler: &mut UpdatableHandler,
        event_list: &mut DemexEventList,
    ) {
        for (_, ts) in self.timecode_slots.iter_mut() {
            ts.update_estimated();
        }

        self.timecodes.values_mut().for_each(|timecode| {
            timecode.update(
                self.timecode_slots
                    .entry(timecode.timecode_slot())
                    .or_default()
                    .estimated_millis(),
                fixture_handler,
                preset_handler,
                updatable_handler,
                event_list,
            )
        });
    }

    fn recalculate_timecode_indices(&mut self, timecode_slot: u32) {
        let Some(ts) = self.timecode_slots.get(&timecode_slot) else {
            return;
        };

        let current_millis = ts.estimated_millis();

        self.timecodes
            .values_mut()
            .filter(|timecode| timecode.timecode_slot() == timecode_slot)
            .for_each(|timecode| {
                timecode
                    .scheduler_mut()
                    .recalculate_next_trigger(current_millis);
            });
    }

    pub fn handle_timecode_packet(
        &mut self,
        timecode_slot: u32,
        timecode_packet: TimedTimecodePacket,
    ) {
        if self
            .timecode_slots
            .entry(timecode_slot)
            .or_default()
            .process_new_timecode(timecode_packet)
        {
            self.recalculate_timecode_indices(timecode_slot);
        }
    }

    pub fn handle_timecode_quarter_frame(
        &mut self,
        timecode_slot: u32,
        piece: MidiQuarterTimecodePiece,
    ) {
        if self
            .timecode_slots
            .entry(timecode_slot)
            .or_default()
            .process_new_quarter_frame(piece)
        {
            self.recalculate_timecode_indices(timecode_slot);
        }
    }
}
