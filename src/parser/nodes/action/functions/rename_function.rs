use serde::{Deserialize, Serialize};

use crate::{
    fixture::timing::TimingHandler,
    parser::nodes::{
        action::{error::ActionRunError, result::ActionRunResult},
        object::{HomeableObject, Object},
    },
};

use super::FunctionArgs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameObjectArgs {
    pub object: Object,
    pub new_name: String,
}

impl FunctionArgs for RenameObjectArgs {
    fn run(
        &self,
        _fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        _fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _: &mut TimingHandler,
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
                HomeableObject::Executor(executor_id) => updatable_handler
                    .rename_executor(*executor_id, self.new_name.clone())
                    .map_err(ActionRunError::UpdatableHandlerError)
                    .map(|_| ActionRunResult::new()),
                HomeableObject::Fader(fader_id) => updatable_handler
                    .rename_fader(*fader_id, self.new_name.clone())
                    .map_err(ActionRunError::UpdatableHandlerError)
                    .map(|_| ActionRunResult::new()),
                HomeableObject::Programmer => Err(ActionRunError::ActionNotImplementedForObject(
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
        }
    }
}
