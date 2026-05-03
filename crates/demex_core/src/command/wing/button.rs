use crate::command::lexer::token::Token;

#[derive(Debug, Clone)]
pub enum CommandWingButton {
    Token(Token),

    Digit(u8),
    Char(char),

    Enter,
}
