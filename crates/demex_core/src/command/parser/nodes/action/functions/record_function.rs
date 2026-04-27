use serde::{Deserialize, Serialize};

use crate::{
    channel3::feature::feature_type::FixtureChannel3FeatureType,
    command::{
        lexer::token::Token,
        parser::{
            error::ParseError,
            nodes::{
                action::{ActionRunArgs, error::ActionRunError, result::ActionRunResult},
                fixture_selector::{FixtureSelector, FixtureSelectorContext},
            },
        },
    },
    event::list::DemexEventList,
    fixture::{Fixture, error::FixtureError},
    patch::Patch,
    presets::{PresetHandler, error::PresetHandlerError, preset::FixturePresetId},
    sequence::cue::{CueFixtureChannelValue, CueIdx},
    state::fixture_state_handler::FixtureStateHandler,
    updatables::error::UpdatableHandlerError,
};

use super::FunctionDelegate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordChannelTypeSelector {
    All,
    Active,
    Features(Vec<FixtureChannel3FeatureType>),
}

impl RecordChannelTypeSelector {
    pub fn get_channel_values(
        &self,
        fixture: &Fixture,
        fixture_state_handler: &FixtureStateHandler,
    ) -> Result<Vec<CueFixtureChannelValue>, FixtureError> {
        let mut values = Vec::new();

        for (attribute, _) in fixture.channel_functions() {
            match self {
                Self::All => {
                    values.push(CueFixtureChannelValue::new(
                        fixture_state_handler
                            .fixture(&fixture.path)
                            .unwrap()
                            .get_programmer_value(attribute)?
                            .clone()
                            .with_started(None),
                        *attribute,
                        false,
                    ));
                }
                Self::Active => {
                    let value = fixture_state_handler
                        .fixture(&fixture.path)
                        .unwrap()
                        .get_programmer_value(attribute)?;

                    if value.is_home() {
                        continue;
                    }

                    values.push(CueFixtureChannelValue::new(
                        value.clone().with_started(None),
                        *attribute,
                        false,
                    ));
                }
                Self::Features(_channels) => {
                    todo!()
                }
            }
        }

        Ok(values)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordPresetArgs {
    pub id: FixturePresetId,
    pub fixture_selector: FixtureSelector,
    pub name: Option<String>,

    /// If true, and the preset already exists and is "Default" or "KeyframeEffect", it will record the next keyframe
    pub should_next: bool,
}

impl FunctionDelegate for RecordPresetArgs {
    fn run(
        &self,
        args: ActionRunArgs,
    ) -> Result<
        crate::command::parser::nodes::action::result::ActionRunResult,
        crate::command::parser::nodes::action::error::ActionRunError,
    > {
        args.preset_handler
            .record_preset(
                &self.fixture_selector,
                args.fixture_selector_context,
                self.id,
                self.name.clone(),
                self.should_next,
                args.patch,
                args.fixture_handler,
                args.timing_handler,
                args.event_list,
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::Default)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordGroupArgs {
    pub id: Option<u32>,
    pub fixture_selector: FixtureSelector,
    pub name: Option<String>,
}

impl FunctionDelegate for RecordGroupArgs {
    fn run(
        &self,
        ActionRunArgs {
            preset_handler,
            fixture_selector_context,
            event_list,
            ..
        }: ActionRunArgs,
    ) -> Result<ActionRunResult, ActionRunError> {
        let selection = self
            .fixture_selector
            .get_selection(preset_handler, fixture_selector_context)
            .map_err(ActionRunError::FixtureSelectorError)?;

        let id = self.id.unwrap_or_else(|| preset_handler.next_group_id());

        preset_handler
            .record_group(selection, id, self.name.clone(), event_list)
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::GroupAdded(id))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordSequenceCueArgs {
    pub sequence_id: u32,
    pub cue_idx: Option<CueIdx>,
    pub fixture_selector: FixtureSelector,
    pub channel_type_selector: RecordChannelTypeSelector,
}

impl FunctionDelegate for RecordSequenceCueArgs {
    fn run(&self, args: ActionRunArgs) -> Result<ActionRunResult, ActionRunError> {
        args.preset_handler
            .record_sequence_cue(
                self.sequence_id,
                args.fixture_handler,
                &self.fixture_selector,
                args.fixture_selector_context,
                self.cue_idx,
                &self.channel_type_selector,
                args.patch,
                args.event_list,
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum RecordSequenceCueShorthandArgsId {
    ExecutorId(u32),
}

impl TryFrom<(Token, u32)> for RecordSequenceCueShorthandArgsId {
    type Error = ParseError;

    fn try_from((token, id): (Token, u32)) -> Result<Self, Self::Error> {
        match token {
            Token::KeywordExecutor => Ok(Self::ExecutorId(id)),
            _ => Err(ParseError::UnexpectedArgs("Expected 'executor'".to_owned())),
        }
    }
}

impl std::fmt::Display for RecordSequenceCueShorthandArgsId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExecutorId(id) => write!(f, "Executor {}", id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordSequenceCueShorthandArgs {
    pub id: RecordSequenceCueShorthandArgsId,
    pub cue_idx: Option<CueIdx>,
    pub fixture_selector: FixtureSelector,
    pub channel_type_selector: RecordChannelTypeSelector,
    pub sequence_name: Option<String>,
}

impl RecordSequenceCueShorthandArgs {
    fn create_sequence(
        &self,
        fixture_handler: &mut FixtureStateHandler,
        preset_handler: &mut PresetHandler,
        fixture_selector_context: FixtureSelectorContext,
        patch: &Patch,
        name: String,
        event_list: &mut DemexEventList,
    ) -> Result<u32, PresetHandlerError> {
        let sequence_id = preset_handler.next_sequence_id();

        preset_handler.create_sequence(sequence_id, Some(name), event_list)?;

        preset_handler.record_sequence_cue(
            sequence_id,
            fixture_handler,
            &self.fixture_selector,
            fixture_selector_context,
            self.cue_idx,
            &self.channel_type_selector,
            patch,
            event_list,
        )?;

        Ok(sequence_id)
    }
}

impl FunctionDelegate for RecordSequenceCueShorthandArgs {
    fn run(&self, args: ActionRunArgs) -> Result<ActionRunResult, ActionRunError> {
        match self.id {
            RecordSequenceCueShorthandArgsId::ExecutorId(executor_id) => {
                if let Ok(executor) = args.updatable_handler.executor_mut(executor_id) {
                    // if the executor is already present, but a sequence name is provided, we want to error
                    if self.sequence_name.is_some() {
                        return Err(ActionRunError::UpdatableHandlerError(
                            UpdatableHandlerError::UpdatableAlreadyExists(executor.id()),
                        ));
                    }

                    args.preset_handler
                        .record_sequence_cue(
                            executor.runtime().sequence_id(),
                            args.fixture_handler,
                            &self.fixture_selector,
                            args.fixture_selector_context,
                            self.cue_idx,
                            &self.channel_type_selector,
                            args.patch,
                            args.event_list,
                        )
                        .map_err(ActionRunError::PresetHandlerError)?;

                    Ok(ActionRunResult::new())
                } else {
                    let sequence_id = self
                        .create_sequence(
                            args.fixture_handler,
                            args.preset_handler,
                            args.fixture_selector_context,
                            args.patch,
                            self.sequence_name.clone().unwrap_or_else(|| {
                                format!("Sequence {}", args.preset_handler.next_sequence_id())
                            }),
                            args.event_list,
                        )
                        .map_err(ActionRunError::PresetHandlerError)?;

                    args.updatable_handler
                        .create_executor(executor_id, sequence_id, args.event_list)
                        .map_err(ActionRunError::UpdatableHandlerError)?;

                    Ok(ActionRunResult::Default)
                }
            }
        }
    }
}
