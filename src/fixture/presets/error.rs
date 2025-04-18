use crate::{
    fixture::{
        channel3::feature::feature_group::FixtureChannel3FeatureGroup, error::FixtureError,
        handler::error::FixtureHandlerError, sequence::cue::CueIdx,
    },
    parser::nodes::{action::error::ActionRunError, fixture_selector::FixtureSelectorError},
};

use super::preset::FixturePresetId;

#[derive(Debug)]
pub enum PresetHandlerError {
    PresetAlreadyExists(u32),
    FeaturePresetAlreadyExists(FixturePresetId),
    PresetNotFound(u32),
    FeaturePresetNotFound(FixturePresetId),
    FeatureGroupMismatch(FixtureChannel3FeatureGroup, FixtureChannel3FeatureGroup),
    FixtureError(FixtureError),
    FixtureHandlerError(FixtureHandlerError),
    FixtureSelectorError(Box<FixtureSelectorError>),
    MacroExecutionError(Box<ActionRunError>),
    CueAlreadyExists(u32, CueIdx),
    CueNotFound(u32, CueIdx),
    CantUpdateNonDefaultCue(u32, CueIdx),
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
            PresetHandlerError::FeaturePresetAlreadyExists(id) => {
                write!(
                    f,
                    "Object with id {} already exists. Use \"update\" to modify or extend it",
                    id
                )
            }
            PresetHandlerError::PresetNotFound(id) => {
                write!(f, "Object with id {} not found", id)
            }
            PresetHandlerError::FeaturePresetNotFound(id) => {
                write!(f, "Object with id {} not found", id)
            }
            PresetHandlerError::FeatureGroupMismatch(g1, g2) => {
                write!(
                    f,
                    "Feature group mismatch: {} and {} are not in the same group",
                    g1.name(),
                    g2.name()
                )
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
            PresetHandlerError::CueNotFound(preset_id, (cue_idx_major, cue_idx_minor)) => {
                write!(
                    f,
                    "Cue {}.{} not found in sequence {}",
                    cue_idx_major, cue_idx_minor, preset_id
                )
            }
            PresetHandlerError::CantUpdateNonDefaultCue(
                sequence_id,
                (cue_idx_major, cue_idx_minor),
            ) => {
                write!(
                    f,
                    "Cue {}.{} in sequence {} is not a default cue and can't be updated",
                    cue_idx_major, cue_idx_minor, sequence_id
                )
            }
            PresetHandlerError::FixtureHandlerError(err) => {
                write!(f, "Fixture handler error: {}", err)
            }
        }
    }
}

impl std::error::Error for PresetHandlerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
