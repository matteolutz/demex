use crate::{
    fixture::{
        error::FixtureError, handler::error::FixtureHandlerError,
        presets::error::PresetHandlerError, timing::error::TimingHandlerError,
        updatables::error::UpdatableHandlerError,
    },
    input::error::DemexInputDeviceError,
    parser::nodes::{
        fixture_selector::FixtureSelectorError,
        object::{Object, ObjectRange},
    },
};

use super::Action;

#[derive(Debug)]
pub enum ActionRunError {
    FixtureHandlerError(FixtureHandlerError),
    FixtureError(FixtureError),
    PresetHandlerError(PresetHandlerError),
    UpdatableHandlerError(UpdatableHandlerError),
    TimingHandlerError(TimingHandlerError),
    InputDeviceError(DemexInputDeviceError),
    FixtureSelectorError(FixtureSelectorError),
    ExecutorIsRunning(u32),
    SequenceDeleteDependencies(u32),
    UnimplementedAction(Action),

    ActionNotImplementedForObject(String, Object),
    ActionNotImplementedForObjectRange(String, ObjectRange),

    Todo(String),
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
            ActionRunError::TimingHandlerError(e) => write!(f, "Timing handler error: {}", e),
            ActionRunError::InputDeviceError(e) => write!(f, "Input device error: {}", e),
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
            ActionRunError::SequenceDeleteDependencies(seq_id) => {
                write!(f, "Sequence with id {} can't be deleted, because there are other objects (executors, faders) referencing it. Delete these first", seq_id)
            }
            ActionRunError::ActionNotImplementedForObject(action, object) => {
                write!(
                    f,
                    "Action {:?} is not implemented for object {:?}",
                    action, object
                )
            }
            ActionRunError::ActionNotImplementedForObjectRange(action, object_range) => {
                write!(
                    f,
                    "Action {:?} is not implemented for object range {:?}",
                    action, object_range
                )
            }
            ActionRunError::Todo(s) => write!(f, "To do: {}", s),
        }
    }
}

impl std::error::Error for ActionRunError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
