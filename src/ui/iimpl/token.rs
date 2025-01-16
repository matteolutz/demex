use crate::{lexer::token::Token, ui::traits::color::IntoEguiColor32};

impl IntoEguiColor32 for &Token {}

impl Into<eframe::egui::Color32> for &Token {
    fn into(self) -> eframe::egui::Color32 {
        match self {
            Token::Numeral(_)
            | Token::FloatingPoint(_)
            | Token::KeywordFixturesSelected
            | Token::KeywordFull
            | Token::KeywordHalf
            | Token::KeywordOut => eframe::egui::Color32::LIGHT_BLUE,
            Token::String(_) => eframe::egui::Color32::LIGHT_GREEN,
            Token::Plus
            | Token::Minus
            | Token::Percent
            | Token::Exclamation
            | Token::ParenOpen
            | Token::ParenClose => eframe::egui::Color32::LIGHT_RED,
            // Action keywords
            Token::KeywordHome
            | Token::KeywordRecord
            | Token::KeywordManSet
            | Token::KeywordRename
            | Token::KeywordClear
            | Token::KeywordIntens
            | Token::KeywordColor
            | Token::KeywordPosition
            | Token::KeywordMacro
            | Token::KeywordCommandSlice
            | Token::KeywordTest => eframe::egui::Color32::GOLD,
            // Other keywords
            Token::KeywordGroup | Token::KeywordThru | Token::KeywordPreset => {
                eframe::egui::Color32::YELLOW
            }
            Token::Eof => eframe::egui::Color32::TRANSPARENT,
        }
    }
}
