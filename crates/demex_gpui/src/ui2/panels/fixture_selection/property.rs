use demex_core::selection::{FixtureSelection, FixtureSelectionProperty};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FixtureSelectionPropertyType {
    Integer,
    Flag,
}

pub trait FixtureSelectionPropertyExt {
    fn get_type(&self) -> FixtureSelectionPropertyType;
    fn get_as_usize(&self, selection: &FixtureSelection) -> usize;
    fn get_as_bool(&self, selection: &FixtureSelection) -> bool;
}

impl FixtureSelectionPropertyExt for FixtureSelectionProperty {
    fn get_type(&self) -> FixtureSelectionPropertyType {
        match self {
            Self::Block | Self::Group | Self::Wings => FixtureSelectionPropertyType::Integer,
            Self::Reverse => FixtureSelectionPropertyType::Flag,
        }
    }

    fn get_as_usize(&self, selection: &FixtureSelection) -> usize {
        match self {
            Self::Block => selection.block(),
            Self::Group => selection.group(),
            Self::Wings => selection.wings(),
            _ => unreachable!(),
        }
    }

    fn get_as_bool(&self, selection: &FixtureSelection) -> bool {
        match self {
            Self::Reverse => selection.reverse(),
            _ => unreachable!(),
        }
    }
}
