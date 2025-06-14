use crate::{
    fixture::presets::error::PresetHandlerError,
    parser::nodes::fixture_selector::FixtureSelectorError,
};

#[derive(Debug)]
pub enum UpdatableHandlerError {
    UpdatableAlreadyExists(u32),
    UpdatableNotFound(u32),
    PresetHandlerError(PresetHandlerError),
    FixtureSelectorError(FixtureSelectorError),
    ExecutorIsNotASequence(u32),
}

impl std::fmt::Display for UpdatableHandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UpdatableHandlerError::UpdatableAlreadyExists(id) => {
                write!(f, "Updatable with id {} already exists", id)
            }
            UpdatableHandlerError::UpdatableNotFound(id) => {
                write!(f, "Updatable id {} not found", id)
            }
            UpdatableHandlerError::FixtureSelectorError(err) => {
                write!(f, "Fixture selector error: {}", err)
            }
            UpdatableHandlerError::PresetHandlerError(err) => {
                write!(f, "Preset handler error: {}", err)
            }
            UpdatableHandlerError::ExecutorIsNotASequence(id) => {
                write!(f, "Executor with id {} is not a sequence", id)
            }
        }
    }
}

impl std::error::Error for UpdatableHandlerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
