use crate::{
    fixture::error::FixtureError,
    parser::nodes::{action::error::ActionRunError, fixture_selector::FixtureSelectorError},
};

#[derive(Debug)]
pub enum PresetHandlerError {
    PresetAlreadyExists(u32),
    PresetNotFound(u32),
    FixtureError(FixtureError),
    FixtureSelectorError(Box<FixtureSelectorError>),
    MacroExecutionError(Box<ActionRunError>),
}

impl std::fmt::Display for PresetHandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PresetHandlerError::PresetAlreadyExists(id) => {
                write!(f, "Preset with id {} already exists", id)
            }
            PresetHandlerError::PresetNotFound(id) => {
                write!(f, "Preset with id {} not found", id)
            }
            PresetHandlerError::FixtureError(err) => write!(f, "{}", err),
            PresetHandlerError::FixtureSelectorError(err) => write!(f, "{}", err),
            PresetHandlerError::MacroExecutionError(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for PresetHandlerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
