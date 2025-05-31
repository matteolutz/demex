use nodes::{
    action::{
        functions::{
            assign_function::{AssignButtonArgs, AssignButtonArgsMode, AssignFaderArgs},
            create_function::{
                CreateEffectPresetArgs, CreateExecutorArgs, CreateMacroArgs, CreateSequenceArgs,
            },
            delete_function::DeleteArgs,
            recall_function::RecallSequenceCueArgs,
            record_function::{
                RecordChannelTypeSelector, RecordGroupArgs, RecordPresetArgs,
                RecordSequenceCueArgs, RecordSequenceCueShorthandArgs,
                RecordSequenceCueShorthandArgsId,
            },
            rename_function::RenameObjectArgs,
            set_function::{SelectionOrSelector, SetFeatureValueArgs, SetFixturePresetArgs},
            update_function::{UpdateMode, UpdatePresetArgs, UpdateSequenceCueArgs},
        },
        ConfigTypeActionData, ValueOrRange,
    },
    object::ObjectRange,
};

use crate::{
    fixture::{
        channel3::feature::feature_type::FixtureChannel3FeatureType,
        presets::preset::FixturePresetId, sequence::cue::CueIdx,
    },
    lexer::token::Token,
};

use self::{
    error::ParseError,
    nodes::{
        action::Action,
        fixture_selector::{AtomicFixtureSelector, FixtureSelector},
        object::{HomeableObject, Object, ObjectTrait},
    },
};

pub mod error;
pub mod nodes;

macro_rules! expect_and_consume_token {
    ($self:ident, $pattern:pat $(if $guard:expr)? $(,)?, $expected:literal) => {
        match $self.current_token()? {
            $pattern $(if $guard)? => $self.advance(),
            unexpected_token => return Err(ParseError::UnexpectedToken(
                unexpected_token.clone(),
                format!("Expected {}", $expected)
            )),
        }
    };
}

pub struct Parser2<'a> {
    tokens: &'a Vec<Token>,
    current_token_idx: usize,
}

impl<'a> Parser2<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Self {
            tokens,
            current_token_idx: 0,
        }
    }

    fn current_token(&self) -> Result<&Token, ParseError> {
        if !self.has_current_token() {
            Err(ParseError::UnexpectedEndOfInput)
        } else {
            Ok(&self.tokens[self.current_token_idx])
        }
    }

    fn has_current_token(&self) -> bool {
        self.current_token_idx < self.tokens.len()
    }

    fn has_next_token(&self) -> bool {
        self.current_token_idx + 1 < self.tokens.len()
    }

    fn advance(&mut self) {
        self.current_token_idx += 1;
    }

    fn try_parse<T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<T, ParseError>,
    ) -> Result<T, ParseError> {
        let current_idx = self.current_token_idx;

        let result = f(self);

        if result.is_err() {
            self.current_token_idx = current_idx;
        }

        result
    }

    fn parse_atomic_fixture_selector(&mut self) -> Result<AtomicFixtureSelector, ParseError> {
        match self.current_token()? {
            &Token::KeywordFixturesSelected => {
                self.advance();
                Ok(AtomicFixtureSelector::CurrentFixturesSelected)
            }
            &Token::Integer(f1) => {
                self.advance();
                match self.current_token()? {
                    &Token::KeywordThru => {
                        self.advance();
                        match self.current_token()? {
                            &Token::Integer(f2) => {
                                self.advance();
                                Ok(AtomicFixtureSelector::FixtureRange(f1, f2))
                            }
                            unexpectd_token => Err(ParseError::UnexpectedToken(
                                unexpectd_token.clone(),
                                "Expected integer".to_owned(),
                            )),
                        }
                    }
                    _ => Ok(AtomicFixtureSelector::SingleFixture(f1)),
                }
            }
            Token::ParenOpen => {
                self.advance();
                let fixture_selector = self.parse_fixture_selector()?;

                match self.current_token()? {
                    &Token::ParenClose => {
                        self.advance();
                        Ok(AtomicFixtureSelector::SelectorGroup(Box::new(
                            fixture_selector,
                        )))
                    }
                    unexpected_token => Err(ParseError::UnexpectedToken(
                        unexpected_token.clone(),
                        "Expected closing parenthesis".to_string(),
                    )),
                }
            }
            Token::KeywordGroup => {
                self.advance();

                match self.current_token()? {
                    &Token::Integer(group_id) => {
                        self.advance();
                        Ok(AtomicFixtureSelector::FixtureGroup(group_id))
                    }
                    unexpected_token => Err(ParseError::UnexpectedToken(
                        unexpected_token.clone(),
                        "Expected integer".to_string(),
                    )),
                }
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec!["\"~\"", "integer", "\"(\"", "\"group\""],
            )),
        }
    }

    pub fn parse_fixture_selector(&mut self) -> Result<FixtureSelector, ParseError> {
        let atomic_selector = self.parse_atomic_fixture_selector()?;

        match self.current_token()? {
            Token::Plus => {
                self.advance();
                let next_selector = self.parse_fixture_selector()?;

                Ok(FixtureSelector::Additive(
                    atomic_selector,
                    Box::new(next_selector),
                ))
            }
            Token::Minus => {
                self.advance();
                let next_selector = self.parse_fixture_selector()?;

                Ok(FixtureSelector::Subtractive(
                    atomic_selector,
                    Box::new(next_selector),
                ))
            }
            Token::Percent => {
                self.advance();

                let inverted = if matches!(self.current_token()?, Token::Exclamation) {
                    self.advance();
                    true
                } else {
                    false
                };

                match self.current_token()? {
                    &Token::Integer(d) => {
                        self.advance();
                        Ok(FixtureSelector::Modulus(atomic_selector, d, inverted))
                    }
                    unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                        unexpected_token.clone(),
                        vec!["integer", "\"!\""],
                    )),
                }
            }
            _ => Ok(FixtureSelector::Atomic(atomic_selector)),
        }
    }

    fn parse_homeable_object(&mut self) -> Result<HomeableObject, ParseError> {
        let fixture_selector = self.try_parse(Self::parse_fixture_selector);

        if let Ok(fixture_selector) = fixture_selector {
            return Ok(HomeableObject::FixtureSelector(fixture_selector));
        }

        if matches!(self.current_token()?, Token::KeywordExecutor) {
            self.advance();
            let executor_id = self.parse_integer()?;

            return Ok(HomeableObject::Executor(executor_id));
        }

        if matches!(self.current_token()?, Token::KeywordProgrammer) {
            self.advance();

            return Ok(HomeableObject::Programmer);
        }

        Err(ParseError::UnexpectedToken(
            self.current_token()?.clone(),
            "Expected homeable object".to_string(),
        ))
    }

    fn parse_object_or_range(&mut self) -> Result<ObjectRange, ParseError> {
        let from_object = self.parse_object()?;

        if matches!(self.current_token()?, Token::KeywordThru) {
            self.advance();

            let to_object = self.parse_object()?;

            ObjectRange::new(from_object, to_object).map_err(ParseError::ObjectError)
        } else {
            Ok(ObjectRange::single(from_object))
        }
    }

    fn parse_object(&mut self) -> Result<Object, ParseError> {
        let homeable_object = self.try_parse(Self::parse_homeable_object);

        if let Ok(object) = homeable_object {
            return Ok(Object::HomeableObject(object));
        }

        if matches!(self.current_token()?, Token::KeywordSequence) {
            self.advance();

            let sequence_id = self.parse_integer()?;

            if matches!(self.current_token()?, Token::KeywordCue) {
                self.advance();

                let cue_idx = self.parse_discrete_cue_idx()?;

                return Ok(Object::SequenceCue(sequence_id, cue_idx));
            }

            return Ok(Object::Sequence(sequence_id));
        }

        if matches!(self.current_token()?, Token::KeywordPreset) {
            self.advance();

            let preset_id = self.parse_preset_id()?;

            return Ok(Object::Preset(preset_id));
        }

        if matches!(self.current_token()?, Token::KeywordMacro) {
            self.advance();

            let macro_id = self.parse_integer()?;

            return Ok(Object::Macro(macro_id));
        }

        Err(ParseError::UnexpectedToken(
            self.current_token()?.clone(),
            "Expected object".to_string(),
        ))
    }

    fn parse_home_function(&mut self) -> Result<Action, ParseError> {
        let object = self.try_parse(Self::parse_homeable_object);

        if let Ok(object) = object {
            Ok(Action::Home(object))
        } else {
            Ok(Action::HomeAll)
        }
    }

    fn parse_discrete_feature_type(&mut self) -> Result<FixtureChannel3FeatureType, ParseError> {
        match self.current_token()? {
            &Token::KeywordIntens => {
                self.advance();
                Ok(FixtureChannel3FeatureType::Dimmer)
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec!["\"intens\""],
            )),
        }
    }

    fn parse_feature_type(&mut self) -> Result<FixtureChannel3FeatureType, ParseError> {
        self.parse_discrete_feature_type()
    }

    fn parse_feature_type_list(&mut self) -> Result<Vec<FixtureChannel3FeatureType>, ParseError> {
        expect_and_consume_token!(self, Token::ParenOpen, "(");

        let mut channel_types = Vec::new();

        if matches!(self.current_token()?, Token::ParenClose) {
            self.advance();
            return Ok(channel_types);
        }

        loop {
            let channel_type = self.parse_feature_type()?;
            channel_types.push(channel_type);

            if matches!(self.current_token()?, Token::ParenClose) {
                self.advance();
                break;
            }

            expect_and_consume_token!(self, Token::Comma, ",");
        }

        Ok(channel_types)
    }

    fn parse_specific_preset(&mut self) -> Result<FixturePresetId, ParseError> {
        expect_and_consume_token!(self, Token::KeywordPreset, "\"preset\"");

        let preset_id = self.parse_preset_id()?;

        Ok(preset_id)
    }

    fn parse_specific_preset_or_range(
        &mut self,
    ) -> Result<ValueOrRange<FixturePresetId>, ParseError> {
        let preset_id = self.parse_specific_preset()?;

        if matches!(self.current_token()?, Token::KeywordThru) {
            self.advance();
            let end_preset_id = self.parse_preset_id()?;
            Ok(ValueOrRange::Thru(preset_id, end_preset_id))
        } else {
            Ok(ValueOrRange::Single(preset_id))
        }
    }

    fn parse_discrete_channel_value_single(&mut self) -> Result<f32, ParseError> {
        match self.current_token()? {
            &Token::KeywordFull => {
                self.advance();
                Ok(1.0)
            }
            &Token::KeywordHalf => {
                self.advance();
                Ok(0.5)
            }
            &Token::KeywordOut => {
                self.advance();
                Ok(0.0)
            }
            &Token::FloatingPoint(value, _) => {
                self.advance();
                Ok(value)
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec!["\"full\"", "\"half\"", "\"out\"", "floating point value"],
            )),
        }
    }

    fn parse_channel_value_single(&mut self) -> Result<ValueOrRange<f32>, ParseError> {
        let value_a = self.parse_discrete_channel_value_single()?;

        if matches!(self.current_token()?, Token::KeywordThru) {
            self.advance();
            let value_b = self.parse_discrete_channel_value_single()?;

            Ok(ValueOrRange::Thru(value_a, value_b))
        } else {
            Ok(ValueOrRange::Single(value_a))
        }
    }

    fn parse_set_function(
        &mut self,
        fixture_selector: FixtureSelector,
    ) -> Result<Action, ParseError> {
        let preset = self.try_parse(Self::parse_specific_preset_or_range);
        if let Ok(preset_id) = preset {
            return Ok(Action::SetFixturePreset(SetFixturePresetArgs {
                selection_or_selector: SelectionOrSelector::Selector(fixture_selector),
                preset_id,
            }));
        }

        let feature_type = self.parse_feature_type()?;

        let feature_value = self.try_parse(Self::parse_channel_value_single);
        if let Ok(feature_value) = feature_value {
            return Ok(Action::SetFeatureValue(SetFeatureValueArgs {
                fixture_selector,
                feature_type,
                feature_value,
            }));
        }

        Err(ParseError::UnexpectedVariant(vec![
            (
                "\"preset\" <channel_type> <id>".to_owned(),
                preset.err().unwrap(),
            ),
            (
                "discrete channel value".to_owned(),
                feature_value.err().unwrap(),
            ),
        ]))
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        match self.current_token()?.clone() {
            Token::String(name) => {
                self.advance();
                Ok(name)
            }
            unexpected_token => Err(ParseError::UnexpectedToken(
                unexpected_token.clone(),
                "Expected string".to_string(),
            )),
        }
    }

    #[allow(dead_code)]
    fn parse_optional_string(&mut self) -> Result<Option<String>, ParseError> {
        match self.current_token()?.clone() {
            Token::String(name) => {
                self.advance();
                Ok(Some(name))
            }
            _ => Ok(None),
        }
    }

    fn parse_as(&mut self) -> Result<String, ParseError> {
        expect_and_consume_token!(self, Token::KeywordAs, "\"as\"");
        let name = self.parse_string()?;
        Ok(name)
    }

    fn parse_integer(&mut self) -> Result<u32, ParseError> {
        match self.current_token()? {
            &Token::Integer(value) => {
                self.advance();
                Ok(value)
            }
            unexpected_token => Err(ParseError::UnexpectedToken(
                unexpected_token.clone(),
                "Expected integer".to_string(),
            )),
        }
    }

    fn parse_float_individual(&mut self) -> Result<(u32, u32), ParseError> {
        match self.current_token()? {
            &Token::FloatingPoint(_, (n, frac)) => {
                self.advance();
                Ok((n, frac))
            }
            unexpected_token => Err(ParseError::UnexpectedToken(
                unexpected_token.clone(),
                "Expected float".to_string(),
            )),
        }
    }

    fn _parse_integer_or_range(&mut self) -> Result<(u32, u32), ParseError> {
        let start = self.parse_integer()?;

        if matches!(self.current_token()?, Token::KeywordThru) {
            self.advance();

            let end = self.parse_integer()?;
            Ok((start, end))
        } else {
            Ok((start, start))
        }
    }

    fn parse_integer_or_next(&mut self) -> Result<Option<u32>, ParseError> {
        match self.current_token()? {
            &Token::Integer(value) => {
                self.advance();
                Ok(Some(value))
            }
            &Token::KeywordNext => {
                self.advance();
                Ok(None)
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec!["integer", "\"next\""],
            )),
        }
    }

    fn parse_discrete_cue_idx(&mut self) -> Result<CueIdx, ParseError> {
        match self.current_token()? {
            &Token::Integer(value) => {
                self.advance();
                Ok((value, 0))
            }
            &Token::FloatingPoint(_, (major, minor)) => {
                self.advance();
                Ok((major, minor))
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec!["cue index (a / a.b)", "\"next\""],
            )),
        }
    }

    fn parse_preset_id(&mut self) -> Result<FixturePresetId, ParseError> {
        match self.current_token()? {
            &Token::FloatingPoint(_, (feature_group_id, preset_id)) => {
                self.advance();
                Ok(FixturePresetId {
                    feature_group: feature_group_id.try_into().map_err(|_| {
                        ParseError::UnexpectedArgs("Expected feature group id".to_owned())
                    })?,
                    preset_id,
                })
            }
            unexpected_token => Err(ParseError::UnexpectedToken(
                unexpected_token.clone(),
                "Expected preset id (feature_group_id.preset_id)".to_string(),
            )),
        }
    }

    fn parse_cue_idx_or_next(&mut self) -> Result<Option<CueIdx>, ParseError> {
        let discrete_cue_idx = self.try_parse(Self::parse_discrete_cue_idx);
        if let Ok(discrete_cue_idx) = discrete_cue_idx {
            return Ok(Some(discrete_cue_idx));
        }

        match self.current_token()? {
            &Token::KeywordNext => {
                self.advance();
                Ok(None)
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec!["cue index (a / a.b)", "\"next\""],
            )),
        }
    }

    fn parse_update_mode(&mut self) -> Result<UpdateMode, ParseError> {
        match self.current_token()? {
            Token::KeywordMerge => {
                self.advance();
                Ok(UpdateMode::Merge)
            }
            Token::KeywordOverride => {
                self.advance();
                Ok(UpdateMode::Override)
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec!["\"merge\"", "\"override\""],
            )),
        }
    }

    fn parse_record_channel_type_selector(
        &mut self,
    ) -> Result<RecordChannelTypeSelector, ParseError> {
        match self.current_token()? {
            Token::KeywordAll => {
                self.advance();
                Ok(RecordChannelTypeSelector::All)
            }
            Token::KeywordActive => {
                self.advance();
                Ok(RecordChannelTypeSelector::Active)
            }
            _ => Ok(RecordChannelTypeSelector::Features(
                self.parse_feature_type_list()?,
            )),
        }
    }

    fn parse_record_function(&mut self) -> Result<Action, ParseError> {
        match self.current_token()? {
            Token::KeywordPreset => {
                self.advance();

                let id = self.parse_preset_id()?;

                expect_and_consume_token!(self, Token::KeywordFor, "\"for\"");

                let fixture_selector = self.parse_fixture_selector()?;

                let preset_name = self.try_parse(Self::parse_as).ok();

                Ok(Action::RecordPreset(RecordPresetArgs {
                    id,
                    fixture_selector,
                    name: preset_name,
                }))
            }
            Token::KeywordGroup => {
                self.advance();

                let id = self.parse_integer_or_next()?;

                expect_and_consume_token!(self, Token::KeywordFor, "\"for\"");

                let fixture_selector = self.parse_fixture_selector()?;

                let group_name = self.try_parse(Self::parse_as).ok();

                Ok(Action::RecordGroup2(RecordGroupArgs {
                    id,
                    fixture_selector,
                    name: group_name,
                }))
            }
            Token::KeywordSequence => {
                self.advance();

                let sequence_id = self.parse_integer()?;

                expect_and_consume_token!(self, Token::KeywordCue, "\"cue\"");

                let cue_idx = self.parse_cue_idx_or_next()?;

                expect_and_consume_token!(self, Token::KeywordFor, "\"for\"");

                let fixture_selector = self.parse_fixture_selector()?;

                let channel_type_selector = if matches!(self.current_token()?, Token::KeywordWith) {
                    self.advance();

                    self.parse_record_channel_type_selector()?
                } else {
                    RecordChannelTypeSelector::Active
                };

                Ok(Action::RecordSequenceCue(RecordSequenceCueArgs {
                    sequence_id,
                    cue_idx,
                    fixture_selector,
                    channel_type_selector,
                }))
            }
            Token::KeywordExecutor => {
                self.advance();

                let id = self.parse_integer()?;

                let cue_idx = self.try_parse(Self::parse_cue_idx_or_next).unwrap_or(None);

                expect_and_consume_token!(self, Token::KeywordFor, "\"for\"");

                let fixture_selector = self.parse_fixture_selector()?;

                let channel_type_selector = if matches!(self.current_token()?, Token::KeywordWith) {
                    self.advance();

                    self.parse_record_channel_type_selector()?
                } else {
                    RecordChannelTypeSelector::Active
                };

                let sequence_name = self.try_parse(Self::parse_as).ok();

                Ok(Action::RecordSequenceCueShorthand(
                    RecordSequenceCueShorthandArgs {
                        id: RecordSequenceCueShorthandArgsId::ExecutorId(id),
                        cue_idx,
                        fixture_selector,
                        channel_type_selector,
                        sequence_name,
                    },
                ))
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec!["\"preset\"", "\"group\"", "\"sequence\"", "\"executor\""],
            )),
        }
    }

    fn parse_rename_function(&mut self) -> Result<Action, ParseError> {
        let object = self.parse_object()?;

        expect_and_consume_token!(self, Token::KeywordTo, "\"to\"");

        let new_name = self.parse_string()?;

        Ok(Action::Rename(RenameObjectArgs { object, new_name }))
    }

    fn parse_create_function(&mut self) -> Result<Action, ParseError> {
        match self.current_token()? {
            Token::KeywordSequence => {
                self.advance();

                let id = self.parse_integer_or_next()?;

                let name = self.try_parse(Self::parse_as).ok();

                Ok(Action::CreateSequence(CreateSequenceArgs { id, name }))
            }
            Token::KeywordExecutor => {
                self.advance();

                let id = self.parse_integer_or_next()?;

                expect_and_consume_token!(self, Token::KeywordFor, "\"for\"");

                expect_and_consume_token!(self, Token::KeywordSequence, "\"sequence\"");

                let sequence_id = self.parse_integer()?;

                Ok(Action::CreateExecutor(CreateExecutorArgs {
                    id,
                    sequence_id,
                }))
            }
            Token::KeywordMacro => {
                self.advance();

                let id = self.parse_integer_or_next()?;

                expect_and_consume_token!(self, Token::KeywordWith, "\"with\"");

                let macro_action = self.parse_command()?;

                let name = self.try_parse(Self::parse_as).ok();

                Ok(Action::CreateMacro(CreateMacroArgs {
                    id,
                    action: Box::new(macro_action),
                    name,
                }))
            }
            Token::KeywordPreset => {
                self.advance();

                let (feature_group_id, preset_id) = self.parse_float_individual()?;

                let name = self.try_parse(Self::parse_as).ok();

                Ok(Action::CreateEffectPreset(CreateEffectPresetArgs {
                    id: FixturePresetId {
                        feature_group: feature_group_id.try_into().map_err(|_| {
                            ParseError::UnexpectedArgs("Expected feature group id".to_owned())
                        })?,
                        preset_id,
                    },
                    name,
                }))
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec!["\"sequence\"", "\"executor\"", "\"macro\"", "\"fader\""],
            )),
        }
    }

    fn parse_update_function(&mut self) -> Result<Action, ParseError> {
        match self.current_token()? {
            Token::KeywordPreset => {
                self.advance();

                let id = self.parse_preset_id()?;

                expect_and_consume_token!(self, Token::KeywordFor, "\"for\"");

                let fixture_selector = self.parse_fixture_selector()?;

                let update_mode = self
                    .try_parse(Self::parse_update_mode)
                    .unwrap_or(UpdateMode::Merge);

                Ok(Action::UpdatePreset(UpdatePresetArgs {
                    id,
                    fixture_selector,
                    update_mode,
                }))
            }
            Token::KeywordSequence | Token::KeywordExecutor => {
                let id_type_keyword = self.current_token()?.clone();

                self.advance();

                let id = self.parse_integer()?;

                expect_and_consume_token!(self, Token::KeywordCue, "\"cue\"");

                let cue_idx = self.parse_discrete_cue_idx()?;

                expect_and_consume_token!(self, Token::KeywordFor, "\"for\"");

                let fixture_selector = self.parse_fixture_selector()?;

                let channel_type_selector = if matches!(self.current_token()?, Token::KeywordWith) {
                    self.advance();

                    self.parse_record_channel_type_selector()?
                } else {
                    RecordChannelTypeSelector::Active
                };

                let update_mode = self
                    .try_parse(Self::parse_update_mode)
                    .unwrap_or(UpdateMode::Merge);

                Ok(Action::UpdateSequenceCue(UpdateSequenceCueArgs {
                    id: (id_type_keyword, id).try_into()?,
                    cue_idx,
                    fixture_selector,
                    channel_type_selector,
                    update_mode,
                }))
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec!["\"preset\"", "\"sequence\""],
            )),
        }
    }

    fn parse_delete_function(&mut self) -> Result<Action, ParseError> {
        /*let action = match self.current_token()? {
            Token::KeywordMacro => {
                self.advance();

                let macro_id_range = self.parse_integer_or_range()?;

                Ok(Action::DeleteMacro(macro_id_range))
            }
            Token::KeywordPreset => {
                // Intentionally not advancing past the keyword,
                // because parse_specific_preset_or_range will consume it
                // self.advance();

                let preset_id_range = self.parse_specific_preset_or_range()?;

                Ok(Action::DeletePreset(preset_id_range))
            }
            Token::KeywordSequence => {
                self.advance();

                let sequence_id_range = self.parse_integer_or_range()?;

                Ok(Action::DeleteSequence(sequence_id_range))
            }
            Token::KeywordExecutor => {
                self.advance();

                let executor_id_range = self.parse_integer_or_range()?;

                Ok(Action::DeleteExecutor(executor_id_range))
            }
            Token::KeywordFader => {
                self.advance();

                let fader_id_range = self.parse_integer_or_range()?;

                Ok(Action::DeleteFader(fader_id_range))
            }
            Token::KeywordGroup => {
                self.advance();

                let group_id_range = self.parse_integer_or_range()?;

                Ok(Action::DeleteGroup(group_id_range))
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec![
                    "\"macro\"",
                    "\"preset\"",
                    "\"sequence\"",
                    "\"executor\"",
                    "\"fader\"",
                    "\"group\"",
                ],
            )),
        };*/

        let object_range = self.parse_object_or_range()?;

        expect_and_consume_token!(self, Token::KeywordReally, "\"really\"");

        Ok(Action::Delete(DeleteArgs { object_range }))
    }

    fn parse_config_function(&mut self) -> Result<Action, ParseError> {
        match self.current_token()? {
            Token::KeywordOutput => {
                self.advance();
                Ok(Action::Config(ConfigTypeActionData::Output))
            }
            Token::KeywordPatch => {
                self.advance();
                Ok(Action::Config(ConfigTypeActionData::Patch))
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec!["\"output\"", "\"patch\""],
            )),
        }
    }

    fn parse_assign_function(&mut self) -> Result<Action, ParseError> {
        if let Ok(fixture_selector) = self.try_parse(Self::parse_fixture_selector) {
            expect_and_consume_token!(self, Token::KeywordTo, "\"to\"");

            let (device_idx, button_id) = self.parse_float_individual()?;

            return Ok(Action::AssignButton(AssignButtonArgs {
                mode: AssignButtonArgsMode::FixtureSelector(fixture_selector),
                device_idx: device_idx as usize,
                button_id,
            }));
        }

        match self.current_token()? {
            Token::KeywordExecutor => {
                self.advance();

                let executor_id = self.parse_integer()?;

                if matches!(self.current_token()?, Token::KeywordFader) {
                    self.advance();

                    expect_and_consume_token!(self, Token::KeywordTo, "\"to\"");

                    let (device_idx, input_fader_id) = self.parse_float_individual()?;

                    Ok(Action::AssignFader(AssignFaderArgs {
                        executor_id,
                        device_idx: device_idx as usize,
                        input_fader_id,
                    }))
                } else {
                    let mode = match self.current_token()? {
                        Token::KeywordGo => {
                            self.advance();
                            Ok(AssignButtonArgsMode::ExecutorGo(executor_id))
                        }
                        Token::KeywordStop => {
                            self.advance();
                            Ok(AssignButtonArgsMode::ExecutorStop(executor_id))
                        }
                        Token::KeywordFlash => {
                            self.advance();

                            let stomp = if matches!(self.current_token()?, Token::KeywordStomp) {
                                self.advance();
                                true
                            } else {
                                false
                            };

                            Ok(AssignButtonArgsMode::ExecutorFlash {
                                id: executor_id,
                                stomp,
                            })
                        }
                        unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                            unexpected_token.clone(),
                            vec!["\"go\"", "\"stop\"", "\"flash\""],
                        )),
                    }?;

                    expect_and_consume_token!(self, Token::KeywordTo, "\"to\"");

                    let (device_idx, button_id) = self.parse_float_individual()?;

                    Ok(Action::AssignButton(AssignButtonArgs {
                        mode,
                        device_idx: device_idx as usize,
                        button_id,
                    }))
                }
            }
            Token::KeywordPreset => {
                // Intentionally not advancing past the keyword,
                // because parse_specific_preset_or_range will consume it
                // self.advance();

                let preset_id = self.parse_specific_preset_or_range()?;

                let fixture_selector = if matches!(self.current_token()?, Token::KeywordWith) {
                    self.advance();
                    Some(self.parse_fixture_selector()?)
                } else {
                    None
                };

                expect_and_consume_token!(self, Token::KeywordTo, "\"to\"");

                let (device_idx, button_id) = self.parse_float_individual()?;

                Ok(Action::AssignButton(AssignButtonArgs {
                    mode: AssignButtonArgsMode::SelectivePreset {
                        preset_id_range: preset_id,
                        fixture_selector,
                    },
                    device_idx: device_idx as usize,
                    button_id,
                }))
            }
            Token::KeywordMacro => {
                self.advance();

                let command = self.parse_command()?;

                expect_and_consume_token!(self, Token::KeywordTo, "\"to\"");

                let (device_idx, button_id) = self.parse_float_individual()?;

                Ok(Action::AssignButton(AssignButtonArgs {
                    mode: AssignButtonArgsMode::Macro(Box::new(command)),
                    device_idx: device_idx as usize,
                    button_id,
                }))
            }
            Token::KeywordTokens => {
                self.advance();

                expect_and_consume_token!(self, Token::KeywordTo, "\"to\"");

                let (device_idx, button_id) = self.parse_float_individual()?;

                expect_and_consume_token!(self, Token::KeywordWith, "\"with\"");

                let mut tokens = Vec::new();
                while !matches!(self.current_token()?, Token::Eof) {
                    tokens.push(self.current_token()?.clone());
                    self.advance();
                }

                Ok(Action::AssignButton(AssignButtonArgs {
                    mode: AssignButtonArgsMode::Tokens(tokens),
                    device_idx: device_idx as usize,
                    button_id,
                }))
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec![
                    "\"executor\"",
                    "\"fader\"",
                    "\"preset\"",
                    "\"macro\"",
                    "\"tokens\"",
                    "fixture selector",
                ],
            )),
        }
    }

    fn parse_unassign_function(&mut self) -> Result<Action, ParseError> {
        match self.current_token()? {
            Token::KeywordButton => {
                self.advance();

                let (device_idx, button_id) = self.parse_float_individual()?;

                Ok(Action::UnassignInputButton {
                    device_idx: device_idx as usize,
                    button_id,
                })
            }
            Token::KeywordFader => {
                self.advance();

                let (device_idx, fader_id) = self.parse_float_individual()?;

                Ok(Action::UnassignInputFader {
                    device_idx: device_idx as usize,
                    fader_id,
                })
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec!["\"button\"", "\"fader\""],
            )),
        }
    }

    fn parse_recall_function(&mut self) -> Result<Action, ParseError> {
        match self.current_token()? {
            Token::KeywordSequence => {
                self.advance();

                let sequence_id = self.parse_integer()?;

                expect_and_consume_token!(self, Token::KeywordCue, "\"\"cue");

                let cue_idx = self.parse_discrete_cue_idx()?;

                Ok(Action::RecallSequenceCue(RecallSequenceCueArgs {
                    sequence_id,
                    cue_idx,
                }))
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec!["\"sequence\""],
            )),
        }
    }

    fn parse_function(&mut self) -> Result<Action, ParseError> {
        if matches!(self.current_token()?, Token::KeywordHome) {
            self.advance();
            return self.parse_home_function();
        }

        if matches!(self.current_token()?, Token::KeywordRecord) {
            self.advance();
            return self.parse_record_function();
        }

        if matches!(self.current_token()?, Token::KeywordCreate) {
            self.advance();
            return self.parse_create_function();
        }

        if matches!(self.current_token()?, Token::KeywordRename) {
            self.advance();
            return self.parse_rename_function();
        }

        if matches!(self.current_token()?, Token::KeywordUpdate) {
            self.advance();
            return self.parse_update_function();
        }

        if matches!(self.current_token()?, Token::KeywordDelete) {
            self.advance();
            return self.parse_delete_function();
        }

        if matches!(self.current_token()?, Token::KeywordAssign) {
            self.advance();
            return self.parse_assign_function();
        }

        if matches!(self.current_token()?, Token::KeywordUnassign) {
            self.advance();
            return self.parse_unassign_function();
        }

        if matches!(self.current_token()?, Token::KeywordClear) {
            self.advance();
            return Ok(Action::ClearAll);
        }

        if matches!(self.current_token()?, Token::KeywordNuzul) {
            self.advance();
            return Ok(Action::Nuzul);
        }

        if matches!(self.current_token()?, Token::KeywordSueud) {
            self.advance();
            return Ok(Action::Sueud);
        }

        if matches!(self.current_token()?, Token::KeywordSave) {
            self.advance();
            return Ok(Action::Save);
        }

        if matches!(self.current_token()?, Token::KeywordConfig) {
            self.advance();
            return self.parse_config_function();
        }

        if matches!(self.current_token()?, Token::KeywordRecall) {
            self.advance();
            return self.parse_recall_function();
        }

        if matches!(self.current_token()?, Token::KeywordTest) {
            self.advance();

            let test_action = self.parse_string()?;

            return Ok(Action::Test(test_action));
        }

        let fixture_selector = self.try_parse(Self::parse_fixture_selector);
        if let Ok(fixture_selector) = fixture_selector {
            return self.parse_set_function(fixture_selector);
        }

        Err(ParseError::UnexpectedTokenAlternatives(
            self.current_token()?.clone(),
            vec![
                "\"home\"",
                "\"record\"",
                "\"create\"",
                "\"rename\"",
                "\"update\"",
                "\"delete\"",
                "\"clear\"",
                "\"save\"",
                "fixture selector",
            ],
        ))
    }

    fn parse_command(&mut self) -> Result<Action, ParseError> {
        let function = self.try_parse(Self::parse_function);

        if let Ok(function) = function {
            return Ok(function);
        }

        let object = self.try_parse(Self::parse_object);

        if let Ok(object) = object {
            return object
                .clone()
                .default_action()
                .ok_or(ParseError::NoDefaultActionForObject(object));
        }

        Err(ParseError::UnexpectedVariant(vec![
            ("function".to_owned(), function.err().unwrap()),
            ("object".to_owned(), object.err().unwrap()),
        ]))
    }

    pub fn parse(&mut self) -> Result<Action, ParseError> {
        let cmd = self.parse_command()?;

        if self.has_next_token() {
            return Err(ParseError::UnexpectedToken(
                self.current_token()?.clone(),
                "Expected EOF".to_string(),
            ));
        }

        Ok(cmd)
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;

    use super::*;

    #[test]
    pub fn test_parser_basic() {
        let mut lexer = Lexer::new("create sequence fader \"My funny sequence\"");

        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser2::new(&tokens);
        let action = parser.parse();

        log::info!("{:?}", action);
    }
}
