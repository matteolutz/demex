use nodes::action::{ConfigTypeActionData, CueIdxSelectorActionData};

use crate::{
    fixture::{
        channel::{
            FixtureChannel, FIXTURE_CHANNEL_COLOR_ID, FIXTURE_CHANNEL_INTENSITY_ID,
            FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
        },
        sequence::cue::CueIdx,
    },
    lexer::token::Token,
    parser::nodes::action::ChannelTypeSelectorActionData,
};

use self::{
    error::ParseError,
    nodes::{
        action::{Action, ChannelValueSingleActionData, UpdateModeActionData},
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

    fn parse_fixture_selector(&mut self) -> Result<FixtureSelector, ParseError> {
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

        if matches!(self.current_token()?, Token::KeywordFader) {
            self.advance();
            let fader_id = self.parse_integer()?;

            return Ok(HomeableObject::Fader(fader_id));
        }

        Err(ParseError::UnexpectedToken(
            self.current_token()?.clone(),
            "Expected homeable object".to_string(),
        ))
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

            let discrete_channel_type = self.parse_discrete_channel_type()?;
            let preset_id = self.parse_integer()?;

            return Ok(Object::Preset(discrete_channel_type, preset_id));
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

    fn parse_discrete_channel_type(&mut self) -> Result<u16, ParseError> {
        match self.current_token()? {
            &Token::KeywordIntens => {
                self.advance();
                Ok(FIXTURE_CHANNEL_INTENSITY_ID)
            }
            &Token::KeywordPosition => {
                self.advance();
                Ok(FIXTURE_CHANNEL_POSITION_PAN_TILT_ID)
            }
            &Token::KeywordColor => {
                self.advance();
                Ok(FIXTURE_CHANNEL_COLOR_ID)
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec!["\"intens\"", "\"color\"", "\"position\""],
            )),
        }
    }

    fn parse_channel_type(&mut self) -> Result<u16, ParseError> {
        let channel_type = self.try_parse(Self::parse_discrete_channel_type);
        if let Ok(channel_type) = channel_type {
            return Ok(channel_type);
        }

        match self.current_token()? {
            &Token::KeywordMaintenance => {
                self.advance();

                match self.current_token()?.clone() {
                    Token::String(channel_name) => {
                        self.advance();

                        Ok(FixtureChannel::get_maintenance_id(
                            channel_name.clone().as_str(),
                        ))
                    }
                    unexpected_token => Err(ParseError::UnexpectedToken(
                        unexpected_token,
                        "Expected string".to_string(),
                    )),
                }
            }
            unexpected_token => Err(ParseError::UnexpectedToken(
                unexpected_token.clone(),
                "Expected channel type".to_string(),
            )),
        }
    }

    fn parse_channel_type_list(&mut self) -> Result<Vec<u16>, ParseError> {
        expect_and_consume_token!(self, Token::ParenOpen, "(");

        let mut channel_types = Vec::new();

        if matches!(self.current_token()?, Token::ParenClose) {
            self.advance();
            return Ok(channel_types);
        }

        loop {
            let channel_type = self.parse_channel_type()?;
            channel_types.push(channel_type);

            if matches!(self.current_token()?, Token::ParenClose) {
                self.advance();
                break;
            }

            expect_and_consume_token!(self, Token::Comma, ",");
        }

        Ok(channel_types)
    }

    fn parse_specific_preset(&mut self) -> Result<(u16, u32), ParseError> {
        expect_and_consume_token!(self, Token::KeywordPreset, "\"preset\"");

        let channel_type = self.parse_discrete_channel_type()?;
        let preset_id = self.parse_integer()?;

        Ok((channel_type, preset_id))
    }

    fn parse_specific_preset_or_range(&mut self) -> Result<(u16, u32, u32), ParseError> {
        let (channel_type, preset_id) = self.parse_specific_preset()?;

        if matches!(self.current_token()?, Token::KeywordThru) {
            self.advance();
            let end_preset_id = self.parse_integer()?;
            Ok((channel_type, preset_id, end_preset_id))
        } else {
            Ok((channel_type, preset_id, preset_id))
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

    fn parse_channel_value_single(&mut self) -> Result<ChannelValueSingleActionData, ParseError> {
        let value_a = self.parse_discrete_channel_value_single()?;

        if matches!(self.current_token()?, Token::KeywordThru) {
            self.advance();
            let value_b = self.parse_discrete_channel_value_single()?;

            Ok(ChannelValueSingleActionData::Thru(value_a, value_b))
        } else {
            Ok(ChannelValueSingleActionData::Single(value_a))
        }
    }

    fn parse_set_function(
        &mut self,
        fixture_selector: FixtureSelector,
    ) -> Result<Action, ParseError> {
        let preset = self.try_parse(Self::parse_specific_preset_or_range);
        if let Ok((channel_type, preset_id_from, preset_id_to)) = preset {
            if preset_id_from == preset_id_to {
                return Ok(Action::SetChannelValuePreset(
                    fixture_selector,
                    channel_type,
                    preset_id_from,
                ));
            }

            return Ok(Action::SetChannelValuePresetRange(
                fixture_selector,
                channel_type,
                preset_id_from,
                preset_id_to,
            ));
        }

        let channel_type = self.parse_channel_type()?;

        let value = self.try_parse(Self::parse_channel_value_single);
        if let Ok(value) = value {
            return Ok(Action::SetChannelValue(
                fixture_selector,
                channel_type,
                value,
            ));
        }

        Err(ParseError::UnexpectedVariant(vec![
            (
                "\"preset\" <channel_type> <id>".to_owned(),
                preset.err().unwrap(),
            ),
            ("discrete channel value".to_owned(), value.err().unwrap()),
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

    fn parse_integer_or_range(&mut self) -> Result<(u32, u32), ParseError> {
        let start = self.parse_integer()?;

        if matches!(self.current_token()?, Token::KeywordThru) {
            self.advance();

            let end = self.parse_integer()?;
            Ok((start, end))
        } else {
            Ok((start, start))
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

    fn parse_cue_idx(&mut self) -> Result<CueIdxSelectorActionData, ParseError> {
        let discrete_cue_idx = self.try_parse(Self::parse_discrete_cue_idx);
        if let Ok(discrete_cue_idx) = discrete_cue_idx {
            return Ok(CueIdxSelectorActionData::Discrete(discrete_cue_idx));
        }

        match self.current_token()? {
            &Token::KeywordNext => {
                self.advance();
                Ok(CueIdxSelectorActionData::Next)
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec!["cue index (a / a.b)", "\"next\""],
            )),
        }
    }

    fn parse_update_mode(&mut self) -> Result<UpdateModeActionData, ParseError> {
        match self.current_token()? {
            Token::KeywordMerge => {
                self.advance();
                Ok(UpdateModeActionData::Merge)
            }
            Token::KeywordOverride => {
                self.advance();
                Ok(UpdateModeActionData::Override)
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec!["\"merge\"", "\"override\""],
            )),
        }
    }

    fn parse_record_function(&mut self) -> Result<Action, ParseError> {
        match self.current_token()? {
            Token::KeywordPreset => {
                self.advance();

                let discrete_channel_type = self.parse_discrete_channel_type()?;
                let id = self.parse_integer()?;

                expect_and_consume_token!(self, Token::KeywordFor, "\"for\"");

                let fixture_selector = self.parse_fixture_selector()?;

                let preset_name = self.try_parse(Self::parse_as).ok();

                Ok(Action::RecordPreset(
                    discrete_channel_type,
                    id,
                    fixture_selector,
                    preset_name,
                ))
            }
            Token::KeywordGroup => {
                self.advance();

                let id = self.parse_integer()?;

                expect_and_consume_token!(self, Token::KeywordFor, "\"for\"");

                let fixture_selector = self.parse_fixture_selector()?;

                let group_name = self.try_parse(Self::parse_as).ok();

                Ok(Action::RecordGroup2(id, fixture_selector, group_name))
            }
            Token::KeywordSequence => {
                self.advance();

                let sequence_id = self.parse_integer()?;

                expect_and_consume_token!(self, Token::KeywordCue, "\"cue\"");

                let cue_idx = self.parse_cue_idx()?;

                expect_and_consume_token!(self, Token::KeywordFor, "\"for\"");

                let fixture_selector = self.parse_fixture_selector()?;

                let channel_type_selector = if matches!(self.current_token()?, Token::KeywordWith) {
                    self.advance();

                    if matches!(self.current_token()?, Token::KeywordAll) {
                        self.advance();
                        ChannelTypeSelectorActionData::All
                    } else {
                        ChannelTypeSelectorActionData::Channels(self.parse_channel_type_list()?)
                    }
                } else {
                    ChannelTypeSelectorActionData::Active
                };

                Ok(Action::RecordSequenceCue(
                    sequence_id,
                    cue_idx,
                    fixture_selector,
                    channel_type_selector,
                ))
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec!["\"preset\"", "\"group\"", "\"sequence\""],
            )),
        }
    }

    fn parse_rename_function(&mut self) -> Result<Action, ParseError> {
        match self.current_token()? {
            Token::KeywordPreset => {
                self.advance();

                let discrete_channel_type = self.parse_discrete_channel_type()?;
                let id = self.parse_integer()?;

                expect_and_consume_token!(self, Token::KeywordTo, "\"to\"");

                let preset_name = self.parse_string()?;

                Ok(Action::RenamePreset(discrete_channel_type, id, preset_name))
            }
            Token::KeywordGroup => {
                self.advance();

                let id = self.parse_integer()?;

                expect_and_consume_token!(self, Token::KeywordTo, "\"to\"");

                let group_name = self.parse_string()?;

                Ok(Action::RenameGroup(id, group_name))
            }
            Token::KeywordSequence => {
                self.advance();

                let id = self.parse_integer()?;

                expect_and_consume_token!(self, Token::KeywordTo, "\"to\"");

                let seq_name = self.parse_string()?;

                Ok(Action::RenameSequence(id, seq_name))
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec!["\"preset\"", "\"group\"", "\"sequence\""],
            )),
        }
    }

    fn parse_create_function(&mut self) -> Result<Action, ParseError> {
        match self.current_token()? {
            Token::KeywordSequence => {
                self.advance();

                let sequence_id = self.parse_integer()?;

                let sequence_name = self.try_parse(Self::parse_as).ok();

                Ok(Action::CreateSequence(sequence_id, sequence_name))
            }
            Token::KeywordExecutor => {
                self.advance();

                let executor_id = self.parse_integer()?;

                expect_and_consume_token!(self, Token::KeywordFor, "\"for\"");
                expect_and_consume_token!(self, Token::KeywordSequence, "\"sequence\"");

                let sequence_id = self.parse_integer()?;

                Ok(Action::CreateExecutor(executor_id, sequence_id))
            }
            Token::KeywordMacro => {
                self.advance();

                let macro_id = self.parse_integer()?;

                expect_and_consume_token!(self, Token::KeywordWith, "\"with\"");

                let command = self.parse_command()?;

                let macro_name = self.try_parse(Self::parse_as).ok();

                Ok(Action::CreateMacro(macro_id, Box::new(command), macro_name))
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec!["\"sequence\"", "\"executor\"", "\"macro\""],
            )),
        }
    }

    fn parse_update_function(&mut self) -> Result<Action, ParseError> {
        match self.current_token()? {
            Token::KeywordPreset => {
                self.advance();

                let preset_channel_type = self.parse_discrete_channel_type()?;
                let preset_id = self.parse_integer()?;

                expect_and_consume_token!(self, Token::KeywordFor, "\"for\"");

                let fixture_selector = self.parse_fixture_selector()?;

                let update_mode = self
                    .try_parse(Self::parse_update_mode)
                    .unwrap_or(UpdateModeActionData::Merge);

                Ok(Action::UpdatePreset(
                    preset_channel_type,
                    preset_id,
                    fixture_selector,
                    update_mode,
                ))
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec!["\"preset\""],
            )),
        }
    }

    fn parse_delete_function(&mut self) -> Result<Action, ParseError> {
        let action = match self.current_token()? {
            Token::KeywordMacro => {
                self.advance();

                let macro_id_range = self.parse_integer_or_range()?;

                Ok(Action::DeleteMacro(macro_id_range))
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec!["\"macro\""],
            )),
        };

        if let Ok(action) = action {
            expect_and_consume_token!(self, Token::KeywordReally, "\"really\"");
            Ok(action)
        } else {
            action
        }
    }

    fn parse_config_function(&mut self) -> Result<Action, ParseError> {
        match self.current_token()? {
            Token::KeywordOutput => {
                self.advance();
                Ok(Action::Config(ConfigTypeActionData::Output))
            }
            unexpected_token => Err(ParseError::UnexpectedTokenAlternatives(
                unexpected_token.clone(),
                vec!["\"output\""],
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

        println!("{:?}", action);
    }
}
