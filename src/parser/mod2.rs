use crate::{
    fixture::channel::{
        FixtureChannel, FIXTURE_CHANNEL_COLOR_ID, FIXTURE_CHANNEL_INTENSITY_ID,
        FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
    },
    lexer::token::Token,
};

use super::{
    error::ParseError,
    nodes::{
        action::{Action, SequenceCreationMode},
        fixture_selector::{AtomicFixtureSelector, FixtureSelector},
        object::{HomeableObject, Object, ObjectTrait},
    },
};

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
            unexpected_token => Err(ParseError::UnexpectedToken(
                unexpected_token.clone(),
                "Expected numeral or \"thru\"".to_string(),
            )),
        }
    }

    fn parse_fixture_selector(&mut self) -> Result<FixtureSelector, ParseError> {
        let atomic_selector = self.parse_atomic_fixture_selector()?;

        match self.current_token()? {
            &Token::Plus => {
                self.advance();
                let next_selector = self.parse_fixture_selector()?;

                Ok(FixtureSelector::Additive(
                    atomic_selector,
                    Box::new(next_selector),
                ))
            }
            &Token::Minus => {
                self.advance();
                let next_selector = self.parse_fixture_selector()?;

                Ok(FixtureSelector::Subtractive(
                    atomic_selector,
                    Box::new(next_selector),
                ))
            }
            &Token::Percent => {
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
                    unexpected_token => Err(ParseError::UnexpectedToken(
                        unexpected_token.clone(),
                        "Expected integer or \"%\"".to_string(),
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
            unexpected_token => Err(ParseError::UnexpectedToken(
                unexpected_token.clone(),
                "Expected discrete channel type".to_string(),
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

    fn parse_specific_preset(&mut self) -> Result<u32, ParseError> {
        match self.current_token()? {
            Token::KeywordPreset => {
                self.advance();

                match self.current_token()? {
                    &Token::Integer(preset_id) => {
                        self.advance();

                        Ok(preset_id)
                    }
                    unexpected_token => Err(ParseError::UnexpectedToken(
                        unexpected_token.clone(),
                        "Expected integer".to_owned(),
                    )),
                }
            }
            unexpected_token => Err(ParseError::UnexpectedToken(
                unexpected_token.clone(),
                "Expected preset".to_owned(),
            )),
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
            &Token::FloatingPoint(value) => {
                self.advance();
                Ok(value)
            }
            unexpected_token => Err(ParseError::UnexpectedToken(
                unexpected_token.clone(),
                "Expected discrete channel value".to_string(),
            )),
        }
    }

    fn parse_set_function(
        &mut self,
        fixture_selector: FixtureSelector,
    ) -> Result<Action, ParseError> {
        let channel_type = self.parse_channel_type()?;

        let preset = self.try_parse(Self::parse_specific_preset);
        if let Ok(preset) = preset {
            return Ok(Action::SetChannelValuePreset(
                fixture_selector,
                channel_type,
                preset,
            ));
        }

        let discrete_value = self.try_parse(Self::parse_discrete_channel_value_single);
        if let Ok(discrete_value) = discrete_value {
            return Ok(Action::SetChannelValue(
                fixture_selector,
                channel_type,
                discrete_value,
            ));
        }

        Err(ParseError::UnexpectedVariant(
            "Expected \"preset <id>\" or discrete channel value".to_owned(),
            vec![preset.err().unwrap(), discrete_value.err().unwrap()],
        ))
    }

    fn parse_optional_string(&mut self) -> Result<Option<String>, ParseError> {
        match self.current_token()?.clone() {
            Token::String(name) => {
                self.advance();
                Ok(Some(name))
            }
            _ => Ok(None),
        }
    }

    fn parse_record_function(&mut self) -> Result<Action, ParseError> {
        match self.current_token()? {
            Token::KeywordPreset => {
                self.advance();

                let discrete_channel_type = self.parse_discrete_channel_type()?;
                let fixture_selector = self.parse_fixture_selector()?;
                let preset_name = self.parse_optional_string()?;

                Ok(Action::RecordPreset(
                    discrete_channel_type,
                    fixture_selector,
                    preset_name,
                ))
            }
            Token::KeywordGroup => {
                self.advance();

                let fixture_selector = self.parse_fixture_selector()?;
                let group_name = self.parse_optional_string()?;

                Ok(Action::RecordGroup2(fixture_selector, group_name))
            }
            unexpected_token => Err(ParseError::UnexpectedToken(
                unexpected_token.clone(),
                "Expected \"preset\"".to_string(),
            )),
        }
    }

    fn parse_create_function(&mut self) -> Result<Action, ParseError> {
        match self.current_token()? {
            Token::KeywordSequence => {
                self.advance();

                let creation_mode = match self.current_token()? {
                    Token::KeywordFader => {
                        self.advance();
                        SequenceCreationMode::Fader
                    }
                    Token::KeywordButton => {
                        self.advance();
                        SequenceCreationMode::Button
                    }
                    _ => {
                        return Err(ParseError::UnexpectedToken(
                            self.current_token()?.clone(),
                            "Expected \"fader\" or \"button\"".to_string(),
                        ))
                    }
                };

                let sequence_name = self.parse_optional_string()?;

                Ok(Action::CreateSequence(creation_mode, sequence_name))
            }
            _ => Err(ParseError::UnexpectedToken(
                self.current_token()?.clone(),
                "Expected \"sequence\"".to_string(),
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

        let fixture_selector = self.try_parse(Self::parse_fixture_selector);
        if let Ok(fixture_selector) = fixture_selector {
            return self.parse_set_function(fixture_selector);
        }

        Err(ParseError::UnexpectedToken(
            self.current_token()?.clone(),
            "Expected function".to_string(),
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

        Err(ParseError::UnexpectedVariant(
            "Expected command (function or object)".to_string(),
            vec![function.err().unwrap(), object.err().unwrap()],
        ))
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
