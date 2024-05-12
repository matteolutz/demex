#[derive(Debug, Clone)]
pub enum Token {
    Numeral(u32),
    KeywordIntens,
    KeywordThru,
    KeywordFull,
    KeywordHome,
}
