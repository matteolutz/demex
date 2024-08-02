#[derive(Debug, Clone)]
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
    KeywordOut,
    KeywordHome,
    KeywordManSet,
    Eof,
}
