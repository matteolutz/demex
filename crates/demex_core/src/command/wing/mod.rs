mod button;
pub use button::*;

use crate::command::{
    lexer::token::Token,
    parser::{Parser2, nodes::action::Action},
};

#[derive(Debug, Clone, Default)]
pub struct CommandWing {
    tokens: Vec<Token>,

    /// All submitted actions
    pending_action_results: Vec<Action>,
}

impl CommandWing {
    pub fn is_active(&self) -> bool {
        !self.tokens.is_empty()
    }
}

impl CommandWing {
    fn current_token_mut(&mut self) -> Option<&mut Token> {
        self.tokens.last_mut()
    }

    fn push_token(&mut self, token: Token) {
        self.tokens.push(token);
    }

    fn submit(&mut self, action: Action) {
        self.pending_action_results.push(action);
        self.tokens.clear();
    }

    fn should_auto_submit(action: &Action) -> bool {
        match action {
            Action::ClearAll => true,
            Action::SetAttributeValue(_) => true,
            _ => false,
        }
    }

    fn parse(&mut self) -> Option<Action> {
        let mut p = Parser2::new(&self.tokens);
        p.parse().ok()
    }
}

impl CommandWing {
    fn handle_digit(&mut self, digit: u8) {
        if let Some(token) = self.current_token_mut() {
            match token {
                Token::Integer(current_int) => {
                    *current_int = (*current_int * 10) + digit as u32;
                    return;
                }
                Token::FloatingPoint(float, (n, frac)) => {
                    // TODO: improve this
                    let mut s = float.to_string();
                    s.push_str(digit.to_string().as_str());

                    *float = s.parse().unwrap();
                    if let Some((new_n, new_fract)) = s.split_once(".") {
                        *n = new_n.parse().unwrap();
                        *frac = new_fract.parse().unwrap();
                    } else {
                        // we don't have a fraction component
                        *n = *float as u32;
                    }

                    return;
                }
                _ => {}
            }
        }

        self.push_token(Token::Integer(digit as u32));
    }

    fn handle_char(&mut self, char: char) {
        if let Some(Token::String(string)) = self.current_token_mut() {
            string.push(char);
            return;
        }

        self.push_token(Token::String(char.to_string()));
    }

    pub fn handle_button_press(&mut self, button: CommandWingButton) {
        match button {
            CommandWingButton::Token(token) => self.push_token(token),
            CommandWingButton::Digit(digit) => self.handle_digit(digit),
            CommandWingButton::Char(char) => self.handle_char(char),
            CommandWingButton::Enter => {
                self.parse().map(|action| self.submit(action));
                return;
            }
        }

        if let Some(action) = self
            .parse()
            .and_then(|action| Self::should_auto_submit(&action).then_some(action))
        {
            self.submit(action);
        }
    }
}
