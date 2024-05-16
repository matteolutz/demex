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
        if self.current_token_idx >= self.tokens.len() {
            Err(ParseError::UnexpectedEndOfInput)
        } else {
            Ok(&self.tokens[self.current_token_idx])
        }
    }

    fn advance(&mut self) {
        self.current_token_idx += 1;
    }

    fn parse_atomic_fixture_selector(&mut self) -> Result<AtomicFixtureSelector, ParseError> {
        let token = self.current_token()?.clone();
        match token {
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

        let fixture_select = self.parse_fixture_selector()?;

        match self.current_token()? {
            Token::KeywordIntens => {
                self.advance();

                let token = self.current_token()?.clone();

                let intensity = match token {
                    Token::Numeral(i) => {
                        self.advance();
                        Ok(i as u8)
                    }
                    Token::KeywordFull => {
                        self.advance();
                        Ok(255)
                    }
                    Token::KeywordOut => {
                        self.advance();
                        Ok(0)
                    }
                    _ => Err(ParseError::UnexpectedToken(
                        token,
                        "Expected numeral, \"full\" or \"out\"".to_string(),
                    )),
                }?;

                Ok(Action::SetIntensity(fixture_select, intensity))
            }
            Token::KeywordManSet => {
                self.advance();

                let token = self.current_token()?.clone();

                match token {
                    Token::String(channel_name) => {
                        self.advance();

                        let token = self.current_token()?.clone();

                        match token {
                            Token::Numeral(i) => {
                                self.advance();
                                Ok(Action::ManSet(fixture_select, channel_name, i as u8))
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
            Token::Eof => {
                self.advance();
                Ok(Action::FixtureSelector(fixture_select))
            }
            _ => Err(ParseError::UnknownAction(format!(
                "{:?}",
                self.current_token()
            ))),
        }
    }

    pub fn parse(&mut self) -> Result<Action, ParseError> {
        self.parse_action()
    }
}
