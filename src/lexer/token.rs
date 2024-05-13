#[derive(Debug, Clone)]
pub enum Token {
    Numeral(u32),
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
}
