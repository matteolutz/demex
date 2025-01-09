use crate::lexer::token::Token;

use self::{
    error::ParseError,
    nodes::{
        action::Action,
        fixture_selector::{AtomicFixtureSelector, FixtureSelector},
    },
};

pub mod error;
pub mod nodes;

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current_token_idx: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Parser<'a> {
        Parser {
            tokens,
            current_token_idx: 0,
        }
    }

    fn current_token(&self) -> Result<&Token, ParseError> {
        if !self.has_next_token() {
            Err(ParseError::UnexpectedEndOfInput)
        } else {
            Ok(&self.tokens[self.current_token_idx])
        }
    }

    fn has_next_token(&self) -> bool {
        self.current_token_idx < self.tokens.len()
    }

    fn advance(&mut self) {
        self.current_token_idx += 1;
    }

    fn parse_atomic_fixture_selector(&mut self) -> Result<AtomicFixtureSelector, ParseError> {
        let token = self.current_token()?.clone();
        match token {
            Token::KeywordFixturesSelected => {
                self.advance();
                Ok(AtomicFixtureSelector::CurrentFixturesSelected)
            }
            Token::Numeral(i1) => {
                self.advance();
                let token = self.current_token()?.clone();

                match token {
                    Token::KeywordThru => {
                        self.advance();
                        let token = self.current_token()?.clone();

                        match token {
                            Token::Numeral(i2) => {
                                self.advance();
                                Ok(AtomicFixtureSelector::FixtureRange(i1, i2))
                            }
                            _ => Err(ParseError::UnexpectedToken(
                                token,
                                "Expected numeral".to_string(),
                            )),
                        }
                    }
                    _ => Ok(AtomicFixtureSelector::SingleFixture(i1)),
                }
            }
            Token::ParenOpen => {
                self.advance();
                let fixture_selector = self.parse_fixture_selector()?;
                let token = self.current_token()?.clone();

                if let Token::ParenClose = token {
                    self.advance();
                    Ok(AtomicFixtureSelector::SelectorGroup(Box::new(
                        fixture_selector,
                    )))
                } else {
                    Err(ParseError::UnexpectedToken(
                        token,
                        "Expected closing parenthesis".to_string(),
                    ))
                }
            }
            Token::KeywordGroup => {
                self.advance();
                let token = self.current_token()?.clone();

                match token {
                    Token::Numeral(i) => {
                        self.advance();
                        Ok(AtomicFixtureSelector::FixtureGroup(i))
                    }
                    _ => Err(ParseError::UnexpectedToken(
                        token,
                        "Expected numeral".to_string(),
                    )),
                }
            }
            _ => Err(ParseError::UnexpectedToken(
                token,
                "Expected numeral or \"thru\"".to_string(),
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
                let current_token = self.current_token()?.clone();
                match current_token {
                    Token::Exclamation => {
                        self.advance();
                        let current_token = self.current_token()?.clone();
                        match current_token {
                            Token::Numeral(d) => {
                                self.advance();
                                Ok(FixtureSelector::Modulus(atomic_selector, d, true))
                            }
                            _ => Err(ParseError::UnexpectedToken(
                                current_token,
                                "Expected numeral".to_string(),
                            )),
                        }
                    }
                    Token::Numeral(d) => {
                        self.advance();
                        Ok(FixtureSelector::Modulus(atomic_selector, d, false))
                    }
                    _ => Err(ParseError::UnexpectedToken(
                        current_token,
                        "Expected numeral or \"!\"".to_string(),
                    )),
                }
            }
            _ => Ok(FixtureSelector::Atomic(atomic_selector)),
        }
    }

    fn parse_action(&mut self) -> Result<Action, ParseError> {
        if let Token::KeywordHome = self.current_token()? {
            self.advance();
            return Ok(Action::GoHomeAll);
        }

        if let Token::KeywordTest = self.current_token()? {
            self.advance();

            let token = self.current_token()?.clone();

            if let Token::String(str) = token {
                return Ok(Action::Test(str));
            } else {
                return Err(ParseError::UnexpectedToken(
                    token,
                    "Expected string".to_string(),
                ));
            }
        }

        if let Token::KeywordClear = self.current_token()? {
            self.advance();
            return Ok(Action::ClearAll);
        }

        if let Token::KeywordRename = self.current_token()? {
            self.advance();

            let token = self.current_token()?.clone();

            match token {
                Token::KeywordGroup => {
                    self.advance();

                    let token = self.current_token()?.clone();

                    if let Token::Numeral(group) = token {
                        self.advance();

                        let token = self.current_token()?.clone();

                        if let Token::String(name) = token {
                            self.advance();
                            return Ok(Action::RenameGroup(group, name));
                        }
                        return Err(ParseError::UnexpectedToken(
                            token,
                            "Expected string".to_string(),
                        ));
                    }
                    return Err(ParseError::UnexpectedToken(
                        token,
                        "Expected numeral".to_string(),
                    ));
                }
                Token::KeywordColor => {
                    self.advance();

                    let token = self.current_token()?.clone();

                    if let Token::KeywordPreset = token {
                        self.advance();

                        let token = self.current_token()?.clone();

                        if let Token::Numeral(group) = token {
                            self.advance();

                            let token = self.current_token()?.clone();

                            if let Token::String(name) = token {
                                self.advance();
                                return Ok(Action::RenameColorPreset(group, name));
                            }
                            return Err(ParseError::UnexpectedToken(
                                token,
                                "Expected string".to_string(),
                            ));
                        }
                        return Err(ParseError::UnexpectedToken(
                            token,
                            "Expected numeral".to_string(),
                        ));
                    } else {
                        return Err(ParseError::UnexpectedToken(
                            token,
                            "Expected \"preset\"".to_string(),
                        ));
                    }
                }
                Token::KeywordPosition => {
                    self.advance();

                    let token = self.current_token()?.clone();

                    if let Token::KeywordPreset = token {
                        self.advance();

                        let token = self.current_token()?.clone();

                        if let Token::Numeral(group) = token {
                            self.advance();

                            let token = self.current_token()?.clone();

                            if let Token::String(name) = token {
                                self.advance();
                                return Ok(Action::RenamePositionPreset(group, name));
                            }
                            return Err(ParseError::UnexpectedToken(
                                token,
                                "Expected string".to_string(),
                            ));
                        }
                        return Err(ParseError::UnexpectedToken(
                            token,
                            "Expected numeral".to_string(),
                        ));
                    } else {
                        return Err(ParseError::UnexpectedToken(
                            token,
                            "Expected \"preset\"".to_string(),
                        ));
                    }
                }
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        token,
                        "Expected \"group\"".to_string(),
                    ))
                }
            }
        }

        let fixture_select = self.parse_fixture_selector()?;

        match self.current_token()? {
            Token::KeywordIntens => {
                self.advance();

                let token = self.current_token()?.clone();

                let intensity = match token {
                    Token::Numeral(i) => {
                        self.advance();
                        Ok(i as f32)
                    }
                    Token::FloatingPoint(i) => {
                        self.advance();
                        Ok(i)
                    }
                    Token::KeywordFull => {
                        self.advance();
                        Ok(100.0)
                    }
                    Token::KeywordOut => {
                        self.advance();
                        Ok(0.0)
                    }
                    _ => Err(ParseError::UnexpectedToken(
                        token,
                        "Expected numeral, \"full\" or \"out\"".to_string(),
                    )),
                }?;

                Ok(Action::SetIntensity(fixture_select, intensity))
            }
            Token::KeywordPosition => {
                self.advance();

                let token = self.current_token()?.clone();

                if matches!(token, Token::Numeral(_) | Token::FloatingPoint(_)) {
                    let pan = (match token {
                        Token::Numeral(i) => i as f32,
                        Token::FloatingPoint(f) => f,
                        _ => unreachable!(),
                    }) / 255.0;

                    self.advance();

                    let token = self.current_token()?.clone();

                    if matches!(token, Token::Numeral(_) | Token::FloatingPoint(_)) {
                        let tilt = (match token {
                            Token::Numeral(i) => i as f32,
                            Token::FloatingPoint(f) => f,
                            _ => unreachable!(),
                        }) / 255.0;

                        Ok(Action::SetPosition(fixture_select, [pan, tilt]))
                    } else {
                        Err(ParseError::UnexpectedToken(
                            token,
                            "Expected numeral or floating point".to_string(),
                        ))
                    }
                } else {
                    match token {
                        Token::KeywordPreset => {
                            self.advance();

                            let token = self.current_token()?.clone();

                            match token {
                                Token::Numeral(preset_id) => {
                                    self.advance();
                                    Ok(Action::SetPositionPreset(fixture_select, preset_id))
                                }
                                _ => Err(ParseError::UnexpectedToken(
                                    token,
                                    "Expected numeral".to_string(),
                                )),
                            }
                        }
                        _ => Err(ParseError::UnexpectedToken(
                            token,
                            "Expected \"preset\"".to_string(),
                        )),
                    }
                }
            }
            Token::KeywordColor => {
                self.advance();

                let token = self.current_token()?.clone();

                // if token is numeral or floating point
                if matches!(token, Token::Numeral(_) | Token::FloatingPoint(_)) {
                    let red = (match token {
                        Token::Numeral(i) => i as f32,
                        Token::FloatingPoint(f) => f,
                        _ => unreachable!(),
                    }) / 255.0;

                    let mut components = vec![red];

                    for _ in 0..3 {
                        self.advance();
                        let token = self.current_token()?.clone();

                        let component = match token {
                            Token::Numeral(i) => Ok(i as f32),
                            Token::FloatingPoint(f) => Ok(f),
                            _ => Err(ParseError::UnexpectedToken(
                                token,
                                "Expected numeral or floating point".to_string(),
                            )),
                        };

                        components.push((component?) / 255.0);
                    }

                    Ok(Action::SetColor(
                        fixture_select,
                        [components[0], components[1], components[2], components[3]],
                    ))
                } else {
                    // TODO: manual color and preset by name
                    match token {
                        Token::KeywordPreset => {
                            self.advance();

                            let token = self.current_token()?.clone();

                            match token {
                                Token::Numeral(preset_id) => {
                                    self.advance();
                                    Ok(Action::SetColorPreset(fixture_select, preset_id))
                                }
                                _ => Err(ParseError::UnexpectedToken(
                                    token,
                                    "Expected numeral".to_string(),
                                )),
                            }
                        }
                        _ => Err(ParseError::UnexpectedToken(
                            token,
                            "Expected \"preset\"".to_string(),
                        )),
                    }
                }
            }
            Token::KeywordManSet => {
                self.advance();

                let token = self.current_token()?.clone();

                match token {
                    Token::String(channel_name) => {
                        self.advance();

                        let token = self.current_token()?.clone();

                        match token {
                            Token::FloatingPoint(i) => {
                                self.advance();
                                Ok(Action::ManSet(fixture_select, channel_name, i))
                            }
                            Token::Numeral(i) => {
                                self.advance();
                                Ok(Action::ManSet(fixture_select, channel_name, i as f32))
                            }
                            _ => Err(ParseError::UnexpectedToken(
                                token,
                                "Expected numeral".to_string(),
                            )),
                        }
                    }
                    _ => Err(ParseError::UnexpectedToken(
                        token,
                        "Expected string".to_string(),
                    )),
                }
            }
            Token::KeywordHome => {
                self.advance();
                Ok(Action::GoHome(fixture_select))
            }
            Token::KeywordRecord => {
                self.advance();

                let token = self.current_token()?.clone();

                match token {
                    Token::KeywordGroup => {
                        self.advance();

                        let token = self.current_token()?.clone();

                        if let Token::Numeral(group) = token {
                            self.advance();
                            Ok(Action::RecordGroup(fixture_select, group))
                        } else {
                            Err(ParseError::UnexpectedToken(
                                token,
                                "Expected numeral".to_string(),
                            ))
                        }
                    }
                    Token::KeywordColor => {
                        self.advance();

                        let token = self.current_token()?.clone();

                        if let Token::Numeral(group) = token {
                            self.advance();
                            Ok(Action::RecordColor(fixture_select, group))
                        } else {
                            Err(ParseError::UnexpectedToken(
                                token,
                                "Expected numeral".to_string(),
                            ))
                        }
                    }
                    Token::KeywordPosition => {
                        self.advance();

                        let token = self.current_token()?.clone();

                        if let Token::Numeral(group) = token {
                            self.advance();
                            Ok(Action::RecordPosition(fixture_select, group))
                        } else {
                            Err(ParseError::UnexpectedToken(
                                token,
                                "Expected numeral".to_string(),
                            ))
                        }
                    }
                    _ => Err(ParseError::UnexpectedToken(
                        token,
                        "Expected \"group\", \"color\" or \"position\"".to_string(),
                    )),
                }
            }

            _ => {
                self.advance();
                Ok(Action::FixtureSelector(fixture_select))
            }
        }
    }

    pub fn parse(&mut self) -> Result<Action, ParseError> {
        let action = self.parse_action()?;

        if !self.has_next_token() {
            return Ok(action);
        }

        if let Token::KeywordRecord = self.current_token()? {
            self.advance();
            let token = self.current_token()?.clone();

            if let Token::KeywordMacro = token {
                self.advance();

                let token = self.current_token()?.clone();

                if let Token::Numeral(i) = token {
                    self.advance();

                    return Ok(Action::RecordMacro(Box::new(action), i));
                } else {
                    return Err(ParseError::UnexpectedToken(
                        token,
                        "Expected numeral".to_string(),
                    ));
                }
            } else {
                return Err(ParseError::UnexpectedToken(
                    token,
                    "Expected \"macro\"".to_string(),
                ));
            }
        };

        if !matches!(self.current_token()?, Token::Eof) {
            return Err(ParseError::UnexpectedToken(
                self.current_token()?.clone(),
                "Expected end of input".to_string(),
            ));
        }

        Ok(action)
    }
}
