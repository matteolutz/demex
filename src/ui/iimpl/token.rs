use crate::{
    lexer::token::{Token, TokenType},
    ui::traits::color::IntoEguiColor32,
};

impl IntoEguiColor32 for &Token {}

impl Into<eframe::egui::Color32> for &Token {
    fn into(self) -> eframe::egui::Color32 {
        match self.token_type() {
            TokenType::Literal | TokenType::ValueKeyword => eframe::egui::Color32::LIGHT_BLUE,
            TokenType::Operator => eframe::egui::Color32::LIGHT_RED,
            TokenType::ActionKeyword => eframe::egui::Color32::LIGHT_RED,
            TokenType::ObjectKeyword => eframe::egui::Color32::YELLOW,
            TokenType::ChannelTypeKeyword => eframe::egui::Color32::LIGHT_GREEN,
            TokenType::OtherKeyword => eframe::egui::Color32::WHITE,
            TokenType::Eof => eframe::egui::Color32::TRANSPARENT,
        }
    }
}
