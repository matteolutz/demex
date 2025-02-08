use crate::{
    fixture::{error::FixtureError, sequence::cue::CueIdx},
    parser::nodes::{action::error::ActionRunError, fixture_selector::FixtureSelectorError},
};

#[derive(Debug)]
pub enum PresetHandlerError {
    PresetAlreadyExists(u32),
    PresetNotFound(u32),
    FixtureError(FixtureError),
    FixtureSelectorError(Box<FixtureSelectorError>),
    MacroExecutionError(Box<ActionRunError>),
    CueAlreadyExists(u32, CueIdx),
}

impl std::fmt::Display for PresetHandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PresetHandlerError::PresetAlreadyExists(id) => {
                write!(
                    f,
                    "Object with id {} already exists. Use \"update\" to modify or extend it",
                    id
                )
            }
            PresetHandlerError::PresetNotFound(id) => {
                write!(f, "Object with id {} not found", id)
            }
            PresetHandlerError::FixtureError(err) => write!(f, "{}", err),
            PresetHandlerError::FixtureSelectorError(err) => write!(f, "{}", err),
            PresetHandlerError::MacroExecutionError(err) => write!(f, "{}", err),
            PresetHandlerError::CueAlreadyExists(preset_id, (cue_idx_major, cue_idx_minor)) => {
                write!(
                    f,
                    "Cue {}.{} already exists in sequence {}. Use \"update\" to modify it",
                    cue_idx_major, cue_idx_minor, preset_id
                )
            }
        }
    }
}

impl std::error::Error for PresetHandlerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
