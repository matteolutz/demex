use std::time;

use serde::{Deserialize, Serialize};

use crate::{
    fixture::{
        channel3::feature::feature_type::FixtureChannel3FeatureType,
        error::FixtureError,
        gdtf::GdtfFixture,
        handler::{FixtureHandler, FixtureTypeList},
        patch::Patch,
        presets::{error::PresetHandlerError, preset::FixturePresetId, PresetHandler},
        sequence::cue::{CueFixtureChannelValue, CueIdx},
        timing::TimingHandler,
        updatables::error::UpdatableHandlerError,
    },
    lexer::token::Token,
    parser::{
        error::ParseError,
        nodes::{
            action::{error::ActionRunError, result::ActionRunResult},
            fixture_selector::{FixtureSelector, FixtureSelectorContext},
        },
    },
};

use super::FunctionArgs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordChannelTypeSelector {
    All,
    Active,
    Features(Vec<FixtureChannel3FeatureType>),
}

impl RecordChannelTypeSelector {
    pub fn get_channel_values(
        &self,
        fixture_types: &FixtureTypeList,
        fixture: &GdtfFixture,
    ) -> Result<Vec<CueFixtureChannelValue>, FixtureError> {
        let mut values = Vec::new();

        let (_, dmx_mode) = fixture.fixture_type_and_dmx_mode(fixture_types)?;

        for dmx_channel in &dmx_mode.dmx_channels {
            match self {
                Self::All => {
                    values.push(CueFixtureChannelValue::new(
                        fixture
                            .get_programmer_value(dmx_channel.name().as_ref())?
                            .clone()
                            .with_preset_state(None),
                        dmx_channel.name().as_ref().to_owned(),
                        false,
                    ));
                }
                Self::Active => {
                    let value = fixture.get_programmer_value(dmx_channel.name().as_ref())?;
                    if value.is_home() {
                        continue;
                    }

                    values.push(CueFixtureChannelValue::new(
                        value.clone().with_preset_state(None),
                        dmx_channel.name().as_ref().to_owned(),
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
}

impl FunctionArgs for RecordPresetArgs {
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
        preset_handler
            .record_preset(
                &self.fixture_selector,
                fixture_selector_context,
                self.id,
                self.name.clone(),
                patch.fixture_types(),
                fixture_handler,
                timing_handler,
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordGroupArgs {
    pub id: Option<u32>,
    pub fixture_selector: FixtureSelector,
    pub name: Option<String>,
}

impl FunctionArgs for RecordGroupArgs {
    fn run(
        &self,
        _issued_at: time::Instant,
        _fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        _updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _: &mut TimingHandler,
        _: &Patch,
    ) -> Result<ActionRunResult, ActionRunError> {
        let selection = self
            .fixture_selector
            .get_selection(preset_handler, fixture_selector_context)
            .map_err(ActionRunError::FixtureSelectorError)?;

        preset_handler
            .record_group(
                selection,
                self.id.unwrap_or_else(|| preset_handler.next_group_id()),
                self.name.clone(),
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordSequenceCueArgs {
    pub sequence_id: u32,
    pub cue_idx: Option<CueIdx>,
    pub fixture_selector: FixtureSelector,
    pub channel_type_selector: RecordChannelTypeSelector,
}

impl FunctionArgs for RecordSequenceCueArgs {
    fn run(
        &self,
        _issued_at: time::Instant,
        fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        _updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _: &mut TimingHandler,
        patch: &Patch,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .record_sequence_cue(
                self.sequence_id,
                fixture_handler,
                &self.fixture_selector,
                fixture_selector_context,
                self.cue_idx,
                &self.channel_type_selector,
                patch.fixture_types(),
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
        fixture_handler: &mut FixtureHandler,
        preset_handler: &mut PresetHandler,
        fixture_selector_context: FixtureSelectorContext,
        fixture_types: &FixtureTypeList,
        name: String,
    ) -> Result<u32, PresetHandlerError> {
        let sequence_id = preset_handler.next_sequence_id();

        preset_handler.create_sequence(sequence_id, Some(name))?;

        preset_handler.record_sequence_cue(
            sequence_id,
            fixture_handler,
            &self.fixture_selector,
            fixture_selector_context,
            self.cue_idx,
            &self.channel_type_selector,
            fixture_types,
        )?;

        Ok(sequence_id)
    }
}

impl FunctionArgs for RecordSequenceCueShorthandArgs {
    fn run(
        &self,
        _issued_at: time::Instant,
        fixture_handler: &mut crate::fixture::handler::FixtureHandler,
        preset_handler: &mut crate::fixture::presets::PresetHandler,
        fixture_selector_context: crate::parser::nodes::fixture_selector::FixtureSelectorContext,
        updatable_handler: &mut crate::fixture::updatables::UpdatableHandler,
        _input_device_handler: &mut crate::input::DemexInputDeviceHandler,
        _timing_handler: &mut TimingHandler,
        patch: &Patch,
    ) -> Result<ActionRunResult, ActionRunError> {
        match self.id {
            RecordSequenceCueShorthandArgsId::ExecutorId(executor_id) => {
                if let Ok(executor) = updatable_handler.executor_mut(executor_id) {
                    // if the executor is already present, but a sequence name is provided, we want to error
                    if self.sequence_name.is_some() {
                        return Err(ActionRunError::UpdatableHandlerError(
                            UpdatableHandlerError::UpdatableAlreadyExists(executor.id()),
                        ));
                    }

                    preset_handler
                        .record_sequence_cue(
                            executor.runtime().sequence_id(),
                            fixture_handler,
                            &self.fixture_selector,
                            fixture_selector_context,
                            self.cue_idx,
                            &self.channel_type_selector,
                            patch.fixture_types(),
                        )
                        .map_err(ActionRunError::PresetHandlerError)?;
                } else {
                    let sequence_id = self
                        .create_sequence(
                            fixture_handler,
                            preset_handler,
                            fixture_selector_context,
                            patch.fixture_types(),
                            self.sequence_name.clone().unwrap_or_else(|| {
                                format!("Sequence {}", preset_handler.next_sequence_id())
                            }),
                        )
                        .map_err(ActionRunError::PresetHandlerError)?;

                    updatable_handler
                        .create_executor(executor_id, sequence_id)
                        .map_err(ActionRunError::UpdatableHandlerError)?;
                }
            }
        }

        Ok(ActionRunResult::new())
    }
}
