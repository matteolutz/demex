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

        match token {
            Token::Numeral(i) => {
                self.advance();
                Ok(FixtureSelector::SingleFixture(i))
            }
            _ => Err(ParseError::UnexpectedToken(
                token,
                "Expected Numeral".to_string(),
            )),
        }
    }

    fn parse_action(&mut self) -> Result<Action, ParseError> {
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
                    _ => Err(ParseError::UnexpectedToken(
                        token,
                        "Expected Numeral".to_string(),
                    )),
                }?;

                Ok(Action::SetIntensity(fixture_select, intensity))
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
