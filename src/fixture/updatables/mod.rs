use std::collections::HashMap;

use error::UpdatableHandlerError;
use executor::Executor;
use fader::{
    config::{DemexFaderConfig, DemexFaderRuntimeFunction},
    DemexFader,
};
use serde::{Deserialize, Serialize};

use crate::parser::nodes::{
    action::functions::create_function::{
        CreateExecutorArgsCreationMode, CreateFaderArgsCreationMode,
    },
    fixture_selector::FixtureSelectorContext,
};

use super::{
    handler::FixtureHandler, presets::PresetHandler, selection::FixtureSelection,
    sequence::runtime::SequenceRuntime, value_source::FixtureChannelValuePriority,
};

pub mod error;
pub mod executor;
pub mod fader;
pub mod runtime;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum StompSource {
    Executor(u32),
    Fader(u32),
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct UpdatableHandler {
    executors: HashMap<u32, Executor>,
    faders: HashMap<u32, DemexFader>,

    #[serde(default, skip_serializing, skip_deserializing)]
    stomps: Vec<StompSource>,
}

impl UpdatableHandler {
    pub fn sequence_deleteable(&mut self, sequence_id: u32) -> bool {
        !(self
            .executors
            .iter()
            .any(|(_, v)| v.refers_to_sequence(sequence_id))
            || self.faders.iter().any(|(_, v)| match v.config() {
                DemexFaderConfig::SequenceRuntime { runtime, .. } => {
                    sequence_id == runtime.sequence_id()
                }
                _ => true,
            }))
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
                    selection,
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
    ) -> Result<(), UpdatableHandlerError> {
        let executor = self
            .executor_mut(id)
            .ok_or(UpdatableHandlerError::UpdatableNotFound(id))?;

        if executor.is_started() {
            executor.next_cue(preset_handler);
            return Ok(());
        }

        let should_stop_others = executor.stop_others();

        if should_stop_others {
            self.executors_stop_all(fixture_handler);
        }

        let executor = self
            .executor_mut(id)
            .ok_or(UpdatableHandlerError::UpdatableNotFound(id))?;
        executor.start(fixture_handler);
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
    ) -> Result<(), UpdatableHandlerError> {
        let executor = self
            .executor_mut(id)
            .ok_or(UpdatableHandlerError::UpdatableNotFound(id))?;

        if executor.is_started() {
            return Ok(());
        }

        let should_stop_others = executor.stop_others();

        if should_stop_others {
            self.executors_stop_all(fixture_handler);
        }

        let executor = self
            .executor_mut(id)
            .ok_or(UpdatableHandlerError::UpdatableNotFound(id))?;
        executor.start(fixture_handler);
        Ok(())
    }

    pub fn stop_executor(
        &mut self,
        id: u32,
        fixture_handler: &mut FixtureHandler,
    ) -> Result<(), UpdatableHandlerError> {
        self.executor_mut(id)
            .ok_or(UpdatableHandlerError::UpdatableNotFound(id))?
            .stop(fixture_handler);
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

    pub fn executors_stop_all(&mut self, fixture_handler: &mut FixtureHandler) {
        for (_, sr) in self.executors.iter_mut() {
            sr.stop(fixture_handler);
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
        delta_time: f64,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
    ) {
        for (_, runtime) in self.executors.iter_mut() {
            runtime.update(delta_time, fixture_handler, preset_handler);
        }
    }

    pub fn delete_executor(&mut self, id: u32) -> Result<(), UpdatableHandlerError> {
        self.executors
            .remove(&id)
            .ok_or(UpdatableHandlerError::UpdatableNotFound(id))?;
        Ok(())
    }
}

// Faders
impl UpdatableHandler {
    pub fn create_fader(
        &mut self,
        id: u32,
        creation_mode: &CreateFaderArgsCreationMode,
        name: Option<String>,
        preset_handler: &PresetHandler,
        fixture_selector_context: FixtureSelectorContext,
    ) -> Result<(), UpdatableHandlerError> {
        if self.faders.contains_key(&id) {
            return Err(UpdatableHandlerError::UpdatableAlreadyExists(id));
        }

        let fader_config = match creation_mode {
            CreateFaderArgsCreationMode::Submaster(fixture_selector) => {
                DemexFaderConfig::Submaster {
                    selection: fixture_selector
                        .get_selection(preset_handler, fixture_selector_context)
                        .map_err(UpdatableHandlerError::FixtureSelectorError)?,
                }
            }
            CreateFaderArgsCreationMode::Sequence(sequence_id, fixture_selector) => {
                preset_handler
                    .get_sequence(*sequence_id)
                    .map_err(UpdatableHandlerError::PresetHandlerError)?;

                let fixtures = fixture_selector
                    .get_selection(preset_handler, fixture_selector_context)
                    .map_err(UpdatableHandlerError::FixtureSelectorError)?;

                DemexFaderConfig::SequenceRuntime {
                    selection: fixtures,
                    runtime: SequenceRuntime::new(*sequence_id),
                    function: DemexFaderRuntimeFunction::default(),
                }
            }
        };

        self.faders.insert(
            id,
            DemexFader::new(
                id,
                name.unwrap_or_else(|| format!("Fader {}", id)),
                fader_config,
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

    pub fn delete_fader(&mut self, id: u32) -> Result<(), UpdatableHandlerError> {
        self.faders
            .remove(&id)
            .ok_or(UpdatableHandlerError::UpdatableNotFound(id))?;
        Ok(())
    }

    pub fn next_fader_id(&self) -> u32 {
        self.faders.keys().max().unwrap_or(&0) + 1
    }

    pub fn rename_fader(&mut self, id: u32, name: String) -> Result<(), UpdatableHandlerError> {
        *self.fader_mut(id)?.name_mut() = name;
        Ok(())
    }
}
