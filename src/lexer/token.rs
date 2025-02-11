use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenType {
    Literal,
    Operator,

    ActionKeyword,
    ObjectKeyword,
    ChannelTypeKeyword,
    ValueKeyword,
    OtherKeyword,

    Eof,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Token {
    Integer(u32),
    FloatingPoint(f32, (u32, u32)),
    String(String),
    Plus,
    Minus,
    Percent,
    Exclamation,
    ParenOpen,
    ParenClose,
    Comma,

    KeywordIntens,
    KeywordColor,
    KeywordPosition,
    KeywordStrobe,
    KeywordMaintenance,

    KeywordThru,
    KeywordFull,
    KeywordHalf,
    KeywordOut,
    KeywordHome,
    KeywordManSet,
    KeywordRecord,
    KeywordCreate,
    KeywordGroup,
    KeywordMacro,
    KeywordCommandSlice,
    KeywordSequence,
    KeywordFader,
    KeywordExecutor,
    KeywordFor,
    KeywordAs,
    KeywordTo,
    KeywordRename,
    KeywordClear,
    KeywordPreset,
    KeywordTest,
    KeywordFixturesSelected,
    KeywordCue,
    KeywordWith,
    KeywordAll,
    KeywordUpdate,
    KeywordMerge,
    KeywordOverride,
    KeywordSave,
    KeywordDelete,
    KeywordReally,
    KeywordDot,
    KeywordNext,
    KeywordConfig,
    KeywordOutput,

    KeywordNuzul,
    KeywordSueud,

    Eof,
}

impl Token {
    pub fn token_type(&self) -> TokenType {
        match self {
            Token::Integer(_) => TokenType::Literal,
            Token::FloatingPoint(_, _) => TokenType::Literal,
            Token::String(_) => TokenType::Literal,
            Token::KeywordFixturesSelected => TokenType::Literal,

            Token::Plus => TokenType::Operator,
            Token::Minus => TokenType::Operator,
            Token::Percent => TokenType::Operator,
            Token::Exclamation => TokenType::Operator,
            Token::ParenOpen => TokenType::Operator,
            Token::ParenClose => TokenType::Operator,
            Token::Comma => TokenType::Operator,

            Token::KeywordIntens => TokenType::ChannelTypeKeyword,
            Token::KeywordColor => TokenType::ChannelTypeKeyword,
            Token::KeywordPosition => TokenType::ChannelTypeKeyword,
            Token::KeywordStrobe => TokenType::ChannelTypeKeyword,
            Token::KeywordMaintenance => TokenType::ChannelTypeKeyword,

            Token::KeywordFull => TokenType::ValueKeyword,
            Token::KeywordHalf => TokenType::ValueKeyword,
            Token::KeywordOut => TokenType::ValueKeyword,

            Token::KeywordHome => TokenType::ActionKeyword,
            Token::KeywordManSet => TokenType::ActionKeyword,
            Token::KeywordRecord => TokenType::ActionKeyword,
            Token::KeywordRename => TokenType::ActionKeyword,
            Token::KeywordClear => TokenType::ActionKeyword,
            Token::KeywordTest => TokenType::ActionKeyword,
            Token::KeywordCreate => TokenType::ActionKeyword,
            Token::KeywordUpdate => TokenType::ActionKeyword,
            Token::KeywordNuzul => TokenType::ActionKeyword,
            Token::KeywordSueud => TokenType::ActionKeyword,
            Token::KeywordSave => TokenType::ActionKeyword,
            Token::KeywordDelete => TokenType::ActionKeyword,
            Token::KeywordConfig => TokenType::ActionKeyword,

            Token::KeywordGroup => TokenType::ObjectKeyword,
            Token::KeywordMacro => TokenType::ObjectKeyword,
            Token::KeywordCommandSlice => TokenType::ObjectKeyword,
            Token::KeywordPreset => TokenType::ObjectKeyword,
            Token::KeywordSequence => TokenType::ObjectKeyword,
            Token::KeywordFader => TokenType::ObjectKeyword,
            Token::KeywordExecutor => TokenType::ObjectKeyword,
            Token::KeywordCue => TokenType::ObjectKeyword,

            Token::KeywordThru => TokenType::OtherKeyword,
            Token::KeywordFor => TokenType::OtherKeyword,
            Token::KeywordAs => TokenType::OtherKeyword,
            Token::KeywordTo => TokenType::OtherKeyword,
            Token::KeywordWith => TokenType::OtherKeyword,
            Token::KeywordAll => TokenType::OtherKeyword,
            Token::KeywordMerge => TokenType::OtherKeyword,
            Token::KeywordOverride => TokenType::OtherKeyword,
            Token::KeywordReally => TokenType::OtherKeyword,
            Token::KeywordDot => TokenType::OtherKeyword,
            Token::KeywordNext => TokenType::OtherKeyword,
            Token::KeywordOutput => TokenType::OtherKeyword,

            Token::Eof => TokenType::Eof,
        }
    }
}

impl Token {
    pub fn quoted(&self) -> String {
        format!("\"{}\"", self)
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Integer(n) => write!(f, "{}", n),
            Token::FloatingPoint(_, (n, frac)) => write!(f, "{}.{}", n, frac),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Percent => write!(f, "%"),
            Token::Exclamation => write!(f, "!"),
            Token::ParenOpen => write!(f, "("),
            Token::ParenClose => write!(f, ")"),
            Token::Comma => write!(f, ","),
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
            Token::KeywordStrobe => write!(f, "strobe"),
            Token::KeywordMaintenance => write!(f, "maintenance"),
            Token::KeywordPreset => write!(f, "preset"),
            Token::KeywordTest => write!(f, "test"),
            Token::KeywordFixturesSelected => write!(f, "~"),
            Token::KeywordCreate => write!(f, "create"),
            Token::KeywordSequence => write!(f, "sequence"),
            Token::KeywordFader => write!(f, "fader"),
            Token::KeywordExecutor => write!(f, "executor"),
            Token::KeywordFor => write!(f, "for"),
            Token::KeywordAs => write!(f, "as"),
            Token::KeywordTo => write!(f, "to"),
            Token::KeywordCue => write!(f, "cue"),
            Token::KeywordWith => write!(f, "with"),
            Token::KeywordAll => write!(f, "all"),
            Token::KeywordUpdate => write!(f, "update"),
            Token::KeywordMerge => write!(f, "merge"),
            Token::KeywordOverride => write!(f, "override"),
            Token::KeywordNuzul => write!(f, "nuzul"),
            Token::KeywordSueud => write!(f, "sueud"),
            Token::KeywordSave => write!(f, "save"),
            Token::KeywordDelete => write!(f, "delete"),
            Token::KeywordReally => write!(f, "really"),
            Token::KeywordDot => write!(f, "dot"),
            Token::KeywordNext => write!(f, "next"),
            Token::KeywordConfig => write!(f, "config"),
            Token::KeywordOutput => write!(f, "output"),
            Token::Eof => write!(f, "Eof"),
        }
    }
}
