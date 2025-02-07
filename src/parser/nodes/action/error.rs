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
        }
    }
}

impl std::error::Error for ActionRunError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
