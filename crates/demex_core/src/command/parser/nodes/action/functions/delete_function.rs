use std::time;

use serde::{Deserialize, Serialize};

use crate::{
    command::parser::nodes::{
        action::{error::ActionRunError, result::ActionRunResult},
        object::{HomeableObject, Object, ObjectRange},
    },
    event::DemexEvent,
    patch::Patch,
    pool::PoolType,
    timing::TimingHandler,
};

use super::FunctionArgs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteArgs {
    pub object_range: ObjectRange,
}

impl FunctionArgs for DeleteArgs {
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
    ) -> Result<
        crate::command::parser::nodes::action::result::ActionRunResult,
        crate::command::parser::nodes::action::error::ActionRunError,
    > {
        match &self.object_range.as_tuple() {
            (Object::Preset(preset_id_from), Object::Preset(preset_id_to)) => {
                // TODO: what happens with data referring to this preset?

                let count = preset_handler
                    .delete_preset_range(*preset_id_from, *preset_id_to)
                    .map_err(ActionRunError::PresetHandlerError)?;

                let result = if self.object_range.is_single() {
                    ActionRunResult::new()
                } else {
                    ActionRunResult::Info(format!("Deleted {} presets", count))
                };

                Ok(result.with_event(DemexEvent::PoolItemsDeleted {
                    // the PresetHandler will make sure from and to feature group match
                    pool_type: PoolType::Preset(preset_id_from.feature_group),
                    from_id: preset_id_from.preset_id,
                    to_id: preset_id_to.preset_id,
                }))
            }
            (Object::Sequence(id_from), Object::Sequence(id_to)) => {
                for id in *id_from..=*id_to {
                    if !updatable_handler.sequence_deleteable(id) {
                        return Err(ActionRunError::SequenceDeleteDependencies(id));
                    }
                }

                for id in *id_from..=*id_to {
                    preset_handler
                        .delete_sequence(id)
                        .map_err(ActionRunError::PresetHandlerError)?;
                }

                let result = if id_from == id_to {
                    ActionRunResult::new()
                } else {
                    ActionRunResult::Info(format!("Deleted {} sequences", id_to - id_from + 1))
                };

                Ok(result.with_event(DemexEvent::PoolItemsDeleted {
                    pool_type: PoolType::Sequence,
                    from_id: *id_from,
                    to_id: *id_to,
                }))
            }
            (
                Object::SequenceCue(sequence_id_from, cue_idx_from),
                Object::SequenceCue(sequence_id_to, cue_idx_to),
            ) => {
                if sequence_id_from != sequence_id_to {
                    return Err(ActionRunError::ActionNotImplementedForObjectRange(
                        "Delete".to_owned(),
                        self.object_range.clone(),
                    ));
                }

                preset_handler
                    .delete_sequence_cues(*sequence_id_from, *cue_idx_from, *cue_idx_to)
                    .map_err(ActionRunError::PresetHandlerError)?;

                if cue_idx_from == cue_idx_to {
                    Ok(ActionRunResult::new())
                } else {
                    Ok(ActionRunResult::Info(format!(
                        "Deleted cue {}.{} to {}.{} in sequence {}",
                        cue_idx_from.0,
                        cue_idx_from.1,
                        cue_idx_to.0,
                        cue_idx_to.1,
                        sequence_id_from
                    )))
                }
            }
            (
                Object::HomeableObject(homeable_object_from),
                Object::HomeableObject(homeable_object_to),
            ) => match (homeable_object_from, homeable_object_to) {
                (
                    HomeableObject::FixtureSelector(fixture_selector_from),
                    HomeableObject::FixtureSelector(fixture_selector_to),
                ) => {
                    let group_id_from = fixture_selector_from.try_as_group_id().ok_or(
                        ActionRunError::ActionNotImplementedForObject(
                            "Delete".to_owned(),
                            self.object_range.from().clone(),
                        ),
                    )?;

                    let group_id_to = fixture_selector_to.try_as_group_id().ok_or(
                        ActionRunError::ActionNotImplementedForObject(
                            "Delete".to_owned(),
                            self.object_range.to().clone(),
                        ),
                    )?;

                    for id in group_id_from..=group_id_to {
                        preset_handler
                            .delete_group(id)
                            .map_err(ActionRunError::PresetHandlerError)?;
                    }

                    let result = if group_id_from == group_id_to {
                        ActionRunResult::new()
                    } else {
                        ActionRunResult::Info(format!(
                            "Deleted {} groups",
                            group_id_from - group_id_to + 1
                        ))
                    };

                    Ok(result.with_event(DemexEvent::PoolItemsDeleted {
                        pool_type: PoolType::Group,
                        from_id: group_id_from,
                        to_id: group_id_to,
                    }))
                }
                (HomeableObject::Executor(id_from), HomeableObject::Executor(id_to)) => {
                    for id in *id_from..=*id_to {
                        if updatable_handler
                            .executor(id)
                            .is_ok_and(|exec| exec.is_active())
                        {
                            return Err(ActionRunError::ExecutorIsRunning(id));
                        }
                    }

                    for id in *id_from..=*id_to {
                        updatable_handler
                            .delete_executor(id)
                            .map_err(ActionRunError::UpdatableHandlerError)?;
                    }

                    let result = if id_from == id_to {
                        ActionRunResult::new()
                    } else {
                        ActionRunResult::Info(format!("Deleted {} executors", id_to - id_from + 1))
                    };

                    Ok(result.with_event(DemexEvent::PoolItemsDeleted {
                        pool_type: PoolType::Executor,
                        from_id: *id_from,
                        to_id: *id_to,
                    }))
                }
                _ => Err(ActionRunError::ActionNotImplementedForObjectRange(
                    "Delete".to_owned(),
                    self.object_range.clone(),
                )),
            },
            _ => Err(ActionRunError::ActionNotImplementedForObjectRange(
                "Delete".to_owned(),
                self.object_range.clone(),
            )),
        }
    }
}
