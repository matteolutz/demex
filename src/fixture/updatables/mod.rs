use std::collections::HashMap;

use error::UpdatableHandlerError;
use fader::{config::DemexFaderRuntimeFunction, DemexFader};
use serde::{Deserialize, Serialize};

use super::{
    handler::{FixtureHandler, FixtureTypeList},
    presets::PresetHandler,
    sequence::runtime::SequenceRuntime,
    timing::TimingHandler,
};

pub mod error;
// pub mod executor;
pub mod fader;
pub mod runtime;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum StompSource {
    Executor(u32),
    Fader(u32),
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UpdatableHandler {
    // executors: HashMap<u32, Executor>,
    faders: HashMap<u32, DemexFader>,

    #[serde(default, skip_serializing, skip_deserializing)]
    stomps: Vec<StompSource>,
}

impl UpdatableHandler {
    pub fn sequence_deleteable(&mut self, sequence_id: u32) -> bool {
        !(/*self
        .executors
        .iter()
        .any(|(_, v)| v.refers_to_sequence(sequence_id))
        ||*/self
            .faders
            .iter()
            .any(|(_, fader)| sequence_id == fader.runtime().sequence_id()))
    }

    pub fn last_stomp_source(&self) -> Option<StompSource> {
        self.stomps.last().cloned()
    }
}

/*
// Executors
impl UpdatableHandler {
    pub fn create_executor(
        &mut self,
        id: u32,
        name: Option<String>,
        creation_mode: &CreateExecutorArgsCreationMode,
        selection: FixtureSelection,
    ) -> Result<(), UpdatableHandlerError> {
        if self.executors.contains_key(&id) {
            return Err(UpdatableHandlerError::UpdatableAlreadyExists(id));
        }

        self.executors.insert(
            id,
            match creation_mode {
                CreateExecutorArgsCreationMode::Effect => Executor::new_effect(
                    id,
                    name,
                    selection,
                    FixtureChannelValuePriority::default(),
                ),
                CreateExecutorArgsCreationMode::Sequence(sequence_id) => Executor::new_sequence(
                    id,
                    name,
                    *sequence_id,
                    FixtureChannelValuePriority::default(),
                ),
            },
        );
        Ok(())
    }

    pub fn executor(&self, id: u32) -> Option<&Executor> {
        self.executors.get(&id)
    }

    pub fn executor_mut(&mut self, id: u32) -> Option<&mut Executor> {
        self.executors.get_mut(&id)
    }

    pub fn start_or_next_executor(
        &mut self,
        id: u32,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
        time_offset: f32,
    ) -> Result<(), UpdatableHandlerError> {
        let executor = self
            .executor_mut(id)
            .ok_or(UpdatableHandlerError::UpdatableNotFound(id))?;

        if executor.is_started() {
            executor.next_cue(fixture_handler, preset_handler, time_offset);
            return Ok(());
        }

        let should_stop_others = executor.stop_others();

        if should_stop_others {
            self.executors_stop_all(fixture_handler, preset_handler);
        }

        let executor = self
            .executor_mut(id)
            .ok_or(UpdatableHandlerError::UpdatableNotFound(id))?;

        executor.start(fixture_handler, preset_handler, time_offset);
        Ok(())
    }

    pub fn executor_stomp(&mut self, id: u32) {
        self.executor_unstomp(id);
        self.stomps.push(StompSource::Executor(id));
    }

    pub fn executor_unstomp(&mut self, id: u32) {
        self.stomps.retain(|v| match v {
            StompSource::Executor(v) => *v != id,
            _ => true,
        });
    }

    pub fn start_executor(
        &mut self,
        id: u32,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
    ) -> Result<(), UpdatableHandlerError> {
        let executor = self
            .executor_mut(id)
            .ok_or(UpdatableHandlerError::UpdatableNotFound(id))?;

        if executor.is_started() {
            return Ok(());
        }

        let should_stop_others = executor.stop_others();

        if should_stop_others {
            self.executors_stop_all(fixture_handler, preset_handler);
        }

        let executor = self
            .executor_mut(id)
            .ok_or(UpdatableHandlerError::UpdatableNotFound(id))?;

        executor.start(fixture_handler, preset_handler, 0.0);
        Ok(())
    }

    pub fn stop_executor(
        &mut self,
        id: u32,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
    ) -> Result<(), UpdatableHandlerError> {
        self.executor_mut(id)
            .ok_or(UpdatableHandlerError::UpdatableNotFound(id))?
            .stop(fixture_handler, preset_handler);
        Ok(())
    }

    pub fn rename_executor(&mut self, id: u32, name: String) -> Result<(), UpdatableHandlerError> {
        *self
            .executor_mut(id)
            .ok_or(UpdatableHandlerError::UpdatableNotFound(id))?
            .name_mut() = name;
        Ok(())
    }

    pub fn executors(&self) -> &HashMap<u32, Executor> {
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

    pub fn next_executor_id(&self) -> u32 {
        self.executors.keys().max().unwrap_or(&0) + 1
    }

    pub fn update_executors(
        &mut self,
        fixture_types: &FixtureTypeList,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) {
        for (_, runtime) in self.executors.iter_mut() {
            runtime.update(
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
}
*/

// Faders
impl UpdatableHandler {
    pub fn create_fader(&mut self, id: u32, sequence_id: u32) -> Result<(), UpdatableHandlerError> {
        if self.faders.contains_key(&id) {
            return Err(UpdatableHandlerError::UpdatableAlreadyExists(id));
        }

        self.faders.insert(
            id,
            DemexFader::new(
                id,
                SequenceRuntime::new(sequence_id),
                DemexFaderRuntimeFunction::default(),
            ),
        );
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

    pub fn faders_stop_all(
        &mut self,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
    ) {
        for (_, fader) in self.faders.iter_mut() {
            fader.stop(fixture_handler, preset_handler);
        }
    }

    pub fn fader_ids(&self) -> Vec<u32> {
        self.faders.keys().cloned().collect()
    }

    pub fn update_faders(
        &mut self,
        fixture_types: &FixtureTypeList,
        fixture_handler: &FixtureHandler,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) {
        for (_, fader) in self.faders.iter_mut() {
            fader.update(
                fixture_types,
                fixture_handler,
                preset_handler,
                timing_handler,
            );
        }
    }

    pub fn delete_fader(&mut self, id: u32) -> Result<(), UpdatableHandlerError> {
        self.faders
            .remove(&id)
            .ok_or(UpdatableHandlerError::UpdatableNotFound(id))?;
        Ok(())
    }

    pub fn next_fader_id(&self) -> u32 {
        self.faders.keys().max().unwrap_or(&0) + 1
    }

    pub fn start_fader(
        &mut self,
        id: u32,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
        time_offset: f32,
    ) -> Result<(), UpdatableHandlerError> {
        self.fader_mut(id)?
            .start(fixture_handler, preset_handler, time_offset);
        Ok(())
    }

    pub fn stop_fader(
        &mut self,
        id: u32,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
    ) -> Result<(), UpdatableHandlerError> {
        self.fader_mut(id)?.stop(fixture_handler, preset_handler);
        Ok(())
    }

    pub fn fader_go(
        &mut self,
        id: u32,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
        time_offset: f32,
    ) -> Result<(), UpdatableHandlerError> {
        self.fader_mut(id)?
            .go(fixture_handler, preset_handler, time_offset);
        Ok(())
    }

    pub fn fader_stomp(&mut self, id: u32) {
        self.fader_unstomp(id);
        self.stomps.push(StompSource::Fader(id));
    }

    pub fn fader_unstomp(&mut self, id: u32) {
        self.stomps.retain(|v| match v {
            StompSource::Fader(v) => *v != id,
            _ => true,
        });
    }
}
