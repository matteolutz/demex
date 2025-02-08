use std::collections::HashMap;

use error::UpdatableHandlerError;
use executor::SequenceRuntimeExecutor;
use fader::DemexFader;
use serde::{Deserialize, Serialize};

use super::{
    channel::value_source::FixtureChannelValuePriority, handler::FixtureHandler,
    presets::PresetHandler,
};

pub mod error;
pub mod executor;
pub mod fader;

#[derive(Serialize, Deserialize, Clone)]
pub struct UpdatableHandler {
    executors: HashMap<u32, SequenceRuntimeExecutor>,
    faders: HashMap<u32, DemexFader>,
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
            SequenceRuntimeExecutor::new(id, sequence_id, FixtureChannelValuePriority::default()),
        );
        Ok(())
    }

    pub fn executor(&self, id: u32) -> Option<&SequenceRuntimeExecutor> {
        self.executors.get(&id)
    }

    pub fn executor_mut(&mut self, id: u32) -> Option<&mut SequenceRuntimeExecutor> {
        self.executors.get_mut(&id)
    }

    pub fn executors(&self) -> &HashMap<u32, SequenceRuntimeExecutor> {
        &self.executors
    }

    pub fn executors_stop_all(
        &mut self,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
    ) {
        for (_, sr) in self.executors.iter_mut() {
            sr.stop(fixture_handler, preset_handler);
        }
    }

    pub fn executor_keys(&self) -> Vec<u32> {
        self.executors.keys().cloned().collect()
    }

    pub fn update_executors(
        &mut self,
        delta_time: f64,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
    ) {
        for (_, runtime) in self.executors.iter_mut() {
            runtime.update(delta_time, fixture_handler, preset_handler);
        }
    }
}

// Faders
impl UpdatableHandler {
    pub fn add_fader(&mut self, fader: DemexFader) -> Result<(), UpdatableHandlerError> {
        if self.faders.contains_key(&fader.id()) {
            return Err(UpdatableHandlerError::UpdatableAlreadyExists(fader.id()));
        }

        self.faders.insert(fader.id(), fader);
        Ok(())
    }

    pub fn fader(&self, id: u32) -> Result<&DemexFader, UpdatableHandlerError> {
        self.faders
            .get(&id)
            .ok_or(UpdatableHandlerError::UpdatableNotFound(id))
    }

    pub fn fader_mut(&mut self, id: u32) -> Result<&mut DemexFader, UpdatableHandlerError> {
        self.faders
            .get_mut(&id)
            .ok_or(UpdatableHandlerError::UpdatableNotFound(id))
    }

    pub fn faders(&self) -> &HashMap<u32, DemexFader> {
        &self.faders
    }

    pub fn faders_home_all(&mut self, fixture_handler: &mut FixtureHandler) {
        for (_, fader) in self.faders.iter_mut() {
            fader.home(fixture_handler);
        }
    }

    pub fn fader_ids(&self) -> Vec<u32> {
        self.faders.keys().cloned().collect()
    }

    pub fn update_faders(&mut self, delta_time: f64, preset_handler: &PresetHandler) {
        for (_, fader) in self.faders.iter_mut() {
            fader.update(delta_time, preset_handler);
        }
    }
}
