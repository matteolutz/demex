use serde::{Deserialize, Serialize};

use crate::command::parser::nodes::{
    action::{ActionRunArgs, error::ActionRunError, result::ActionRunResult},
    object::{HomeableObject, Object, ObjectRange},
};

use super::FunctionDelegate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteArgs {
    pub object_range: ObjectRange,
}

impl FunctionDelegate for DeleteArgs {
    fn run(
        &self,
        ActionRunArgs {
            preset_handler,
            updatable_handler,
            event_list,
            ..
        }: ActionRunArgs,
    ) -> Result<
        crate::command::parser::nodes::action::result::ActionRunResult,
        crate::command::parser::nodes::action::error::ActionRunError,
    > {
        match &self.object_range.as_tuple() {
            (Object::Preset(preset_id_from), Object::Preset(preset_id_to)) => {
                // TODO: what happens with data referring to this preset?

                let count = preset_handler
                    .delete_preset_range(*preset_id_from, *preset_id_to, event_list)
                    .map_err(ActionRunError::PresetHandlerError)?;

                let result = if self.object_range.is_single() {
                    ActionRunResult::new()
                } else {
                    ActionRunResult::Info(format!("Deleted {} presets", count))
                };

                Ok(result)
            }
            (Object::Sequence(id_from), Object::Sequence(id_to)) => {
                for id in *id_from..=*id_to {
                    if !updatable_handler.sequence_deleteable(id) {
                        return Err(ActionRunError::SequenceDeleteDependencies(id));
                    }
                }

                for id in *id_from..=*id_to {
                    preset_handler
                        .delete_sequence(id, event_list)
                        .map_err(ActionRunError::PresetHandlerError)?;
                }

                let result = if id_from == id_to {
                    ActionRunResult::new()
                } else {
                    ActionRunResult::Info(format!("Deleted {} sequences", id_to - id_from + 1))
                };

                Ok(result)
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
                    .delete_sequence_cues(*sequence_id_from, *cue_idx_from, *cue_idx_to, event_list)
                    .map_err(ActionRunError::PresetHandlerError)?;

                if cue_idx_from == cue_idx_to {
                    Ok(ActionRunResult::new())
                } else {
                    Ok(ActionRunResult::Info(format!(
                        "Deleted cue {} to {} in sequence {}",
                        cue_idx_from, cue_idx_to, sequence_id_from
                    )))
                }
            }
            (
                Object::ExecutorCue(executor_id_from, cue_idx_from),
                Object::ExecutorCue(executor_id_to, cue_idx_to),
            ) => {
                if executor_id_from != executor_id_to {
                    return Err(ActionRunError::ActionNotImplementedForObjectRange(
                        "Delete".to_owned(),
                        self.object_range.clone(),
                    ));
                }

                let sequence_id = updatable_handler
                    .executor(*executor_id_from)
                    .map(|executor| executor.runtime().sequence_id())
                    .map_err(ActionRunError::UpdatableHandlerError)?;

                preset_handler
                    .delete_sequence_cues(sequence_id, *cue_idx_from, *cue_idx_to, event_list)
                    .map_err(ActionRunError::PresetHandlerError)?;

                if cue_idx_from == cue_idx_to {
                    Ok(ActionRunResult::new())
                } else {
                    Ok(ActionRunResult::Info(format!(
                        "Deleted cue {} to {} in sequence {}",
                        cue_idx_from, cue_idx_to, sequence_id
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
                            .delete_group(id, event_list)
                            .map_err(ActionRunError::PresetHandlerError)?;
                    }

                    /*
                    let result = if group_id_from == group_id_to {
                        ActionRunResult::new()
                    } else {
                        ActionRunResult::Info(format!(
                            "Deleted {} groups",
                            group_id_from - group_id_to + 1
                        ))
                    };
                    */

                    Ok(ActionRunResult::GroupsRemoved(
                        (group_id_from..=group_id_to).collect(),
                    ))
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
                            .delete_executor(id, preset_handler, event_list)
                            .map_err(ActionRunError::UpdatableHandlerError)?;
                    }

                    let result = if id_from == id_to {
                        ActionRunResult::new()
                    } else {
                        ActionRunResult::Info(format!("Deleted {} executors", id_to - id_from + 1))
                    };

                    Ok(result)
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
