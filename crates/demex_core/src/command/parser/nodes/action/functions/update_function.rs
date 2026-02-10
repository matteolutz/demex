use serde::{Deserialize, Serialize};

use crate::{
    command::{
        lexer::token::Token,
        parser::{
            error::ParseError,
            nodes::{
                action::{ActionRunArgs, error::ActionRunError, result::ActionRunResult},
                fixture_selector::FixtureSelector,
            },
        },
    },
    presets::preset::FixturePresetId,
    sequence::cue::CueIdx,
};

use super::{FunctionDelegate, record_function::RecordChannelTypeSelector};

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

impl FunctionDelegate for UpdatePresetArgs {
    fn run(
        &self,
        args: ActionRunArgs,
    ) -> Result<
        crate::command::parser::nodes::action::result::ActionRunResult,
        crate::command::parser::nodes::action::error::ActionRunError,
    > {
        let num_updated = args
            .preset_handler
            .update_preset(
                &self.fixture_selector,
                args.fixture_selector_context,
                self.id,
                args.patch,
                args.fixture_handler,
                args.timing_handler,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePresetGlobalArgs {
    pub id: FixturePresetId,
}

impl FunctionDelegate for UpdatePresetGlobalArgs {
    fn run(&self, args: ActionRunArgs) -> Result<ActionRunResult, ActionRunError> {
        args.preset_handler
            .make_preset_global(self.id, args.event_list)
            .map_err(ActionRunError::PresetHandlerError)?;
        Ok(ActionRunResult::new())
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

impl FunctionDelegate for UpdateSequenceCueArgs {
    fn run(&self, args: ActionRunArgs) -> Result<ActionRunResult, ActionRunError> {
        let sequence_id = match self.id {
            UpdateSequenceCueArgsId::SequenceId(id) => id,
            UpdateSequenceCueArgsId::ExecutorId(id) => args
                .updatable_handler
                .executor(id)
                .map(|executor| executor.runtime().sequence_id())
                .map_err(ActionRunError::UpdatableHandlerError)?,
        };

        let num_updated = args
            .preset_handler
            .update_sequence_cue(
                sequence_id,
                self.cue_idx,
                &self.fixture_selector,
                args.fixture_selector_context,
                args.fixture_handler,
                &self.channel_type_selector,
                self.update_mode,
                args.patch,
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
