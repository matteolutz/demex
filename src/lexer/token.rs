use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Token {
    Numeral(u32),
    FloatingPoint(f32),
    String(String),
    Plus,
    Minus,
    Percent,
    Exclamation,
    ParenOpen,
    ParenClose,
    KeywordIntens,
    KeywordThru,
    KeywordFull,
    KeywordHalf,
    KeywordOut,
    KeywordHome,
    KeywordManSet,
    KeywordRecord,
    KeywordGroup,
    KeywordMacro,
    KeywordCommandSlice,
    KeywordRename,
    KeywordClear,
    KeywordColor,
    KeywordPosition,
    KeywordPreset,
    KeywordTest,
    KeywordFixturesSelected,
    Eof,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Numeral(n) => write!(f, "{}", n),
            Token::FloatingPoint(n) => write!(f, "{}", n),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Percent => write!(f, "%"),
            Token::Exclamation => write!(f, "!"),
            Token::ParenOpen => write!(f, "("),
            Token::ParenClose => write!(f, ")"),
            Token::KeywordIntens => write!(f, "intes"),
            Token::KeywordThru => write!(f, "thru"),
            Token::KeywordFull => write!(f, "full"),
            Token::KeywordHalf => write!(f, "half"),
            Token::KeywordOut => write!(f, "out"),
            Token::KeywordHome => write!(f, "home"),
            Token::KeywordManSet => write!(f, "manset"),
            Token::KeywordRecord => write!(f, "record"),
            Token::KeywordGroup => write!(f, "group"),
            Token::KeywordMacro => write!(f, "macro"),
            Token::KeywordCommandSlice => write!(f, "commandslice"),
            Token::KeywordRename => write!(f, "rename"),
            Token::KeywordClear => write!(f, "clear"),
            Token::KeywordColor => write!(f, "color"),
            Token::KeywordPosition => write!(f, "position"),
            Token::KeywordPreset => write!(f, "preset"),
            Token::KeywordTest => write!(f, "test"),
            Token::KeywordFixturesSelected => write!(f, "~"),
            Token::Eof => write!(f, "Eof"),
        }
    }
}
