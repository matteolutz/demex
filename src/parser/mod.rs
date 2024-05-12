use crate::lexer::token::Token;

use self::{
    error::ParseError,
    nodes::{action::Action, fixture_selector::FixtureSelector},
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

    fn current_token(&self) -> &Token {
        &self.tokens[self.current_token_idx]
    }

    fn advance(&mut self) {
        self.current_token_idx += 1;
    }

    fn parse_fixture_selector(&mut self) -> Result<FixtureSelector, ParseError> {
        let token = self.current_token().clone();

        if let Token::Numeral(i1) = token {
            self.advance();
            if let Token::KeywordThru = self.current_token() {
                self.advance();

                let token = self.current_token().clone();

                if let Token::Numeral(i2) = token {
                    self.advance();
                    return Ok(FixtureSelector::FixtureRange(i1, i2));
                } else {
                    return Err(ParseError::UnexpectedToken(
                        token,
                        "Expected Numeral".to_string(),
                    ));
                }
            } else {
                return Ok(FixtureSelector::SingleFixture(i1));
            }
        }

        Err(ParseError::UnexpectedToken(
            token,
            "Expected Numeral".to_string(),
        ))
    }

    fn parse_action(&mut self) -> Result<Action, ParseError> {
        if let Token::KeywordHome = self.current_token() {
            self.advance();
            return Ok(Action::GoHomeAll);
        }

        let fixture_select = self.parse_fixture_selector()?;

        let action = match self.current_token() {
            Token::KeywordIntens => {
                self.advance();

                let token = self.current_token().clone();

                let intensity = match token {
                    Token::Numeral(i) => {
                        self.advance();
                        Ok(i as u8)
                    }
                    Token::KeywordFull => {
                        self.advance();
                        Ok(255)
                    }
                    _ => Err(ParseError::UnexpectedToken(
                        token,
                        "Expected Numeral or keyword \"full\"".to_string(),
                    )),
                }?;

                Ok(Action::SetIntensity(fixture_select, intensity))
            }
            Token::KeywordHome => {
                self.advance();
                Ok(Action::GoHome(fixture_select))
            }
            _ => Err(ParseError::UnknownAction(format!(
                "{:?}",
                self.current_token()
            ))),
        };

        action
    }

    pub fn parse(&mut self) -> Result<Action, ParseError> {
        self.parse_action()
    }
}
