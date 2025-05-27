use std::time;

use serde::{Deserialize, Serialize};

use crate::{
    fixture::{
        patch::Patch, presets::preset::FixturePresetId, sequence::cue::CueIdx,
        timing::TimingHandler,
    },
    lexer::token::Token,
    parser::{
        error::ParseError,
        nodes::{
            action::{error::ActionRunError, result::ActionRunResult},
            fixture_selector::FixtureSelector,
        },
    },
};

use super::{record_function::RecordChannelTypeSelector, FunctionArgs};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UpdateMode {
    Merge,
    Override,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePresetArgs {
    pub id: FixturePresetId,
    pub fixture_selector: FixtureSelector,
    pub update_mode: UpdateMode,
}

impl FunctionArgs for UpdatePresetArgs {
    fn run(
        &self,
        _issued_at: time::Instant,
        fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        _updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        timing_handler: &mut TimingHandler,
        patch: &Patch,
    ) -> Result<
        crate::parser::nodes::action::result::ActionRunResult,
        crate::parser::nodes::action::error::ActionRunError,
    > {
        let num_updated = preset_handler
            .update_preset(
                &self.fixture_selector,
                fixture_selector_context,
                self.id,
                patch.fixture_types(),
                fixture_handler,
                timing_handler,
                self.update_mode,
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        if num_updated == 0 {
            Ok(ActionRunResult::Warn(
                "No fixtures we're affected. If you're trying override existing preset data, try running with the \"override\" flag.".to_owned(),
            ))
        } else {
            Ok(ActionRunResult::new())
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum UpdateSequenceCueArgsId {
    SequenceId(u32),
    ExecutorId(u32),
}

impl TryFrom<(Token, u32)> for UpdateSequenceCueArgsId {
    type Error = ParseError;

    fn try_from((token, id): (Token, u32)) -> Result<Self, Self::Error> {
        match token {
            Token::KeywordSequence => Ok(UpdateSequenceCueArgsId::SequenceId(id)),
            Token::KeywordExecutor => Ok(UpdateSequenceCueArgsId::ExecutorId(id)),
            _ => Err(ParseError::UnexpectedArgs(
                "Expected 'sequence', 'executor'".to_owned(),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSequenceCueArgs {
    pub id: UpdateSequenceCueArgsId,
    pub cue_idx: CueIdx,
    pub fixture_selector: FixtureSelector,
    pub channel_type_selector: RecordChannelTypeSelector,
    pub update_mode: UpdateMode,
}

impl FunctionArgs for UpdateSequenceCueArgs {
    fn run(
        &self,
        _issued_at: time::Instant,
        fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _: &mut TimingHandler,
        patch: &Patch,
    ) -> Result<ActionRunResult, ActionRunError> {
        let sequence_id = match self.id {
            UpdateSequenceCueArgsId::SequenceId(id) => id,
            UpdateSequenceCueArgsId::ExecutorId(id) => updatable_handler
                .executor(id)
                .map(|executor| executor.runtime().sequence_id())
                .map_err(ActionRunError::UpdatableHandlerError)?,
        };

        let num_updated = preset_handler
            .update_sequence_cue(
                sequence_id,
                self.cue_idx,
                &self.fixture_selector,
                fixture_selector_context,
                fixture_handler,
                &self.channel_type_selector,
                self.update_mode,
                patch.fixture_types(),
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        if num_updated == 0 {
            Ok(ActionRunResult::Warn(
                "No fixtures we're affected. If you're trying override existing preset data, try running with the \"override\" flag.".to_owned(),
            ))
        } else {
            Ok(ActionRunResult::new())
        }
    }
}
