use std::collections::HashMap;

use error::UpdatableHandlerError;
use serde::{Deserialize, Serialize};

use super::{
    handler::FixtureHandler,
    presets::{fader::DemexFader, PresetHandler},
    sequence::preset::SequenceRuntimePreset,
};

pub mod error;

#[derive(Serialize, Deserialize)]
pub struct UpdatableHandler {
    sequence_runtimes: HashMap<u32, SequenceRuntimePreset>,
    faders: HashMap<u32, DemexFader>,
}

// Sequence Runtimes
impl UpdatableHandler {
    pub fn add_sequence_runtime(&mut self, runtime: SequenceRuntimePreset) {
        self.sequence_runtimes.insert(runtime.id(), runtime);
    }

    pub fn sequence_runtime(&self, id: u32) -> Option<&SequenceRuntimePreset> {
        self.sequence_runtimes.get(&id)
    }

    pub fn sequence_runtime_mut(&mut self, id: u32) -> Option<&mut SequenceRuntimePreset> {
        self.sequence_runtimes.get_mut(&id)
    }

    pub fn sequence_runtimes(&self) -> &HashMap<u32, SequenceRuntimePreset> {
        &self.sequence_runtimes
    }

    pub fn sequence_runtimes_stop_all(
        &mut self,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
    ) {
        for (_, sr) in self.sequence_runtimes.iter_mut() {
            sr.stop(fixture_handler, preset_handler);
        }
    }

    pub fn sequence_runtime_keys(&self) -> Vec<u32> {
        self.sequence_runtimes.keys().cloned().collect()
    }

    pub fn update_sequence_runtimes(
        &mut self,
        delta_time: f64,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
    ) {
        for (_, runtime) in self.sequence_runtimes.iter_mut() {
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
