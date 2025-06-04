use std::collections::HashMap;

use error::UpdatableHandlerError;
use executor::{fader_function::DemexExecutorFaderFunction, DemexExecutor};
use serde::{Deserialize, Serialize};

use super::{
    handler::{FixtureHandler, FixtureTypeList},
    presets::PresetHandler,
    sequence::runtime::SequenceRuntime,
    timing::TimingHandler,
};

pub mod error;
pub mod executor;
pub mod runtime;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum StompSource {
    Executor(u32),
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UpdatableHandler {
    executors: HashMap<u32, DemexExecutor>,

    #[serde(default, skip_serializing, skip_deserializing)]
    stomps: Vec<StompSource>,
}

impl UpdatableHandler {
    pub fn sequence_deleteable(&mut self, sequence_id: u32) -> bool {
        !(self
            .executors
            .iter()
            .any(|(_, fader)| sequence_id == fader.runtime().sequence_id()))
    }

    pub fn last_stomp_source(&self) -> Option<StompSource> {
        self.stomps.last().cloned()
    }
}

// Executors
impl UpdatableHandler {
    pub fn create_executor(
        &mut self,
        id: u32,
        sequence_id: u32,
    ) -> Result<(), UpdatableHandlerError> {
        if self.executors.contains_key(&id) {
            return Err(UpdatableHandlerError::UpdatableAlreadyExists(id));
        }

        self.executors.insert(
            id,
            DemexExecutor::new(
                id,
                SequenceRuntime::new(sequence_id),
                DemexExecutorFaderFunction::default(),
            ),
        );
        Ok(())
    }

    pub fn executor(&self, id: u32) -> Result<&DemexExecutor, UpdatableHandlerError> {
        self.executors
            .get(&id)
            .ok_or(UpdatableHandlerError::UpdatableNotFound(id))
    }

    pub fn executor_mut(&mut self, id: u32) -> Result<&mut DemexExecutor, UpdatableHandlerError> {
        self.executors
            .get_mut(&id)
            .ok_or(UpdatableHandlerError::UpdatableNotFound(id))
    }

    pub fn executors(&self) -> &HashMap<u32, DemexExecutor> {
        &self.executors
    }

    pub fn executors_stop_all(
        &mut self,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
    ) {
        for (_, fader) in self.executors.iter_mut() {
            fader.stop(fixture_handler, preset_handler);
        }
    }

    pub fn executor_ids(&self) -> Vec<u32> {
        self.executors.keys().cloned().collect()
    }

    pub fn update_executors(
        &mut self,
        fixture_types: &FixtureTypeList,
        fixture_handler: &FixtureHandler,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) {
        for (_, fader) in self.executors.iter_mut() {
            fader.update(
                fixture_types,
                fixture_handler,
                preset_handler,
                timing_handler,
            );
        }
    }

    pub fn delete_executor(&mut self, id: u32) -> Result<(), UpdatableHandlerError> {
        self.executors
            .remove(&id)
            .ok_or(UpdatableHandlerError::UpdatableNotFound(id))?;
        Ok(())
    }

    pub fn next_executor_id(&self) -> u32 {
        self.executors.keys().max().unwrap_or(&0) + 1
    }

    pub fn start_executor(
        &mut self,
        id: u32,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
        time_offset: f32,
    ) -> Result<(), UpdatableHandlerError> {
        self.executor_mut(id)?
            .start(fixture_handler, preset_handler, time_offset);
        Ok(())
    }

    pub fn stop_executor(
        &mut self,
        id: u32,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
    ) -> Result<(), UpdatableHandlerError> {
        self.executor_mut(id)?.stop(fixture_handler, preset_handler);
        Ok(())
    }

    pub fn executor_go(
        &mut self,
        id: u32,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
        time_offset: f32,
    ) -> Result<(), UpdatableHandlerError> {
        self.executor_mut(id)?
            .go(fixture_handler, preset_handler, time_offset);
        Ok(())
    }

    pub fn executor_stomp(&mut self, id: u32) {
        self.executor_unstomp(id);
        self.stomps.push(StompSource::Executor(id));
    }

    pub fn executor_unstomp(&mut self, id: u32) {
        self.stomps.retain(|v| match v {
            StompSource::Executor(v) => *v != id,
        });
    }
}
