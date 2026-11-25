use strum::IntoEnumIterator;

use crate::command::parser::nodes::object::ObjectType;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ExpectedParseSlice {
    FaderId { is_unassign: bool },
    ButtonId { is_unassign: bool },

    Object(ObjectType),
    FixtureAttribute,
}

impl ExpectedParseSlice {
    pub fn any_object() -> impl Iterator<Item = Self> {
        ObjectType::iter().map(Self::Object)
    }
}

impl std::fmt::Display for ExpectedParseSlice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ExpectedParseSlice::FaderId { .. } => write!(f, "FaderId"),
            ExpectedParseSlice::ButtonId { .. } => write!(f, "ButtonId"),
            ExpectedParseSlice::Object(object_type) => write!(f, "{:?}", object_type),
            ExpectedParseSlice::FixtureAttribute => write!(f, "FixtureAttribute"),
        }
    }
}
