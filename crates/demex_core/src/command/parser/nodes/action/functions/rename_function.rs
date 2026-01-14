use std::time;

use serde::{Deserialize, Serialize};

use crate::{
    command::parser::nodes::{
        action::{error::ActionRunError, result::ActionRunResult},
        object::{HomeableObject, Object},
    },
    patch::Patch,
    timing::TimingHandler,
};

use super::FunctionArgs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameObjectArgs {
    pub object: Object,
    pub new_name: String,
}

// TODO: use set property instead of rename
impl FunctionArgs for RenameObjectArgs {
    fn run(
        &self,
        _issued_at: time::Instant,
        _fixture_handler: &mut crate::state::fixture_state_handler::FixtureStateHandler,
        preset_handler: &mut crate::presets::PresetHandler,
        _fixture_selector_context: crate::command::parser::nodes::fixture_selector::FixtureSelectorContext,
        updatable_handler: &mut crate::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _: &mut TimingHandler,
        _: &Patch,
        _: &mut crate::event::list::DemexEventList,
    ) -> Result<ActionRunResult, ActionRunError> {
        match &self.object {
            Object::Preset(preset_id) => preset_handler
                .rename_preset(*preset_id, self.new_name.clone())
                .map_err(ActionRunError::PresetHandlerError)
                .map(|_| ActionRunResult::new()),
            Object::Sequence(sequence_id) => preset_handler
                .rename_sequence(*sequence_id, self.new_name.clone())
                .map_err(ActionRunError::PresetHandlerError)
                .map(|_| ActionRunResult::new()),
            Object::HomeableObject(homeable_object) => match homeable_object {
                HomeableObject::FixtureSelector(fixture_selector) => fixture_selector
                    .try_as_group_id()
                    .ok_or(ActionRunError::ActionNotImplementedForObject(
                        "Rename".to_owned(),
                        self.object.clone(),
                    ))
                    .and_then(|group_id| {
                        preset_handler
                            .rename_group(group_id, self.new_name.clone())
                            .map_err(ActionRunError::PresetHandlerError)
                    })
                    .map(|_| ActionRunResult::new()),
                HomeableObject::Programmer => Err(ActionRunError::ActionNotImplementedForObject(
                    "Rename".to_owned(),
                    self.object.clone(),
                )),
                _ => Err(ActionRunError::ActionNotImplementedForObject(
                    "Rename".to_owned(),
                    self.object.clone(),
                )),
            },
            Object::Macro(id) => preset_handler
                .rename_macro(*id, self.new_name.clone())
                .map_err(ActionRunError::PresetHandlerError)
                .map(|_| ActionRunResult::new()),

            Object::SequenceCue(sequence_id, cue_idx) => preset_handler
                .rename_sequence_cue(*sequence_id, *cue_idx, self.new_name.clone())
                .map_err(ActionRunError::PresetHandlerError)
                .map(|_| ActionRunResult::new()),

            Object::ExecutorCue(executor_id, cue_idx) => updatable_handler
                .executor(*executor_id)
                .map_err(ActionRunError::UpdatableHandlerError)
                .and_then(|executor| {
                    let sequence_id = executor.runtime().sequence_id();

                    preset_handler
                        .rename_sequence_cue(sequence_id, *cue_idx, self.new_name.clone())
                        .map_err(ActionRunError::PresetHandlerError)
                        .map(|_| ActionRunResult::new())
                }),
        }
    }
}
