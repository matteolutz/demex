#[derive(Debug, Clone)]
pub enum Token {
    Numeral(u32),
    Plus,
    Minus,
    ParenOpen,
    ParenClose,
    KeywordIntens,
    KeywordThru,
    KeywordFull,
    KeywordOut,
    KeywordHome,
}
