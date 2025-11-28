use crate::{
    lexer::token::{Token, TokenType},
    ui::traits::color::IntoEguiColor32,
};

impl IntoEguiColor32 for &Token {}

impl Into<ecolor::Color32> for &Token {
    fn into(self) -> ecolor::Color32 {
        match self.token_type() {
            TokenType::Literal | TokenType::ValueKeyword => ecolor::Color32::LIGHT_BLUE,
            TokenType::Operator => ecolor::Color32::LIGHT_RED,
            TokenType::ActionKeyword => ecolor::Color32::LIGHT_RED,
            TokenType::ObjectKeyword => ecolor::Color32::YELLOW,
            TokenType::ChannelTypeKeyword => ecolor::Color32::LIGHT_GREEN,
            TokenType::OtherKeyword => ecolor::Color32::WHITE,
            TokenType::Eof => ecolor::Color32::TRANSPARENT,
        }
    }
}
