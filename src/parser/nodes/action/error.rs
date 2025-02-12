use crate::{
    fixture::{
        error::FixtureError, handler::error::FixtureHandlerError,
        presets::error::PresetHandlerError, updatables::error::UpdatableHandlerError,
    },
    parser::nodes::fixture_selector::FixtureSelectorError,
};

use super::Action;

#[derive(Debug)]
pub enum ActionRunError {
    FixtureHandlerError(FixtureHandlerError),
    FixtureError(FixtureError),
    PresetHandlerError(PresetHandlerError),
    UpdatableHandlerError(UpdatableHandlerError),
    FixtureSelectorError(FixtureSelectorError),
    ExecutorIsRunning(u32),
    FaderIsActive(u32),
    SequenceDeleteDependencies(u32),
    UnimplementedAction(Action),
}

impl std::fmt::Display for ActionRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionRunError::FixtureHandlerError(e) => write!(f, "Fixture handler error: {}", e),
            ActionRunError::FixtureError(e) => write!(f, "Fixture error: {}", e),
            ActionRunError::PresetHandlerError(e) => write!(f, "Preset handler error: {}", e),
            ActionRunError::UpdatableHandlerError(e) => {
                write!(f, "Updatable handler error: {}", e)
            }
            ActionRunError::FixtureSelectorError(e) => write!(f, "Fixture selector error: {}", e),
            ActionRunError::UnimplementedAction(action) => {
                write!(f, "Unimplemented action: {:?}", action)
            }
            ActionRunError::ExecutorIsRunning(executor_id) => {
                write!(
                    f,
                    "Executor with id {} is running. Please stop it before trying to modify it",
                    executor_id
                )
            }
            ActionRunError::FaderIsActive(fader_id) => {
                write!(
                    f,
                    "Fader with id {} is not in home position. Please move it to home position before trying to modify it",
                    fader_id
                )
            }
            ActionRunError::SequenceDeleteDependencies(seq_id) => {
                write!(f, "Sequence with id {} can't be deleted, because there are other objects (executors, faders) referencing it. Delete these first", seq_id)
            }
        }
    }
}

impl std::error::Error for ActionRunError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
