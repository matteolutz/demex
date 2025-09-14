#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ExpectedParseSlice {
    FaderId { is_unassign: bool },
    ButtonId { is_unassign: bool },
}

impl std::fmt::Display for ExpectedParseSlice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ExpectedParseSlice::FaderId { .. } => write!(f, "FaderId"),
            ExpectedParseSlice::ButtonId { .. } => write!(f, "ButtonId"),
        }
    }
}
