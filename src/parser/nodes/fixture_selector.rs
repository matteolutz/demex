use serde::{Deserialize, Serialize};

use crate::fixture::{
    presets::{error::PresetHandlerError, PresetHandler},
    selection::FixtureSelection,
};

#[derive(Debug, Clone)]
pub struct FixtureSelectorContext<'a> {
    current_fixture_selection: &'a Option<FixtureSelection>,
}

impl<'a> FixtureSelectorContext<'a> {
    pub fn new(current_fixture_selection: &'a Option<FixtureSelection>) -> Self {
        Self {
            current_fixture_selection,
        }
    }
}

#[derive(Debug)]
pub enum FixtureSelectorError {
    PresetHandlerError(PresetHandlerError),
    FailedToFlatten(Box<FixtureSelectorError>),
    NoFixturesMatched,
    SomeFixturesFailedToMatch(usize),
}

impl std::fmt::Display for FixtureSelectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::PresetHandlerError(e) => write!(f, "PresetHandlerError: {}", e),
            Self::FailedToFlatten(e) => write!(f, "Failed to flatten fixture selector: {}", e),
            Self::NoFixturesMatched => write!(f, "No fixtures matched the given selector"),
            Self::SomeFixturesFailedToMatch(num_fixtures) => {
                write!(f, "{} fixtures failed to match", num_fixtures)
            }
        }
    }
}

impl std::error::Error for FixtureSelectorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AtomicFixtureSelector {
    SingleFixture(u32),
    FixtureRange(u32, u32),
    FixtureGroup(u32),
    SelectorGroup(Box<FixtureSelector>),
    FixtureIdList(Vec<u32>),
    CurrentFixturesSelected,
    None,
}

impl AtomicFixtureSelector {
    // TOOD: change this to get_selection
    pub fn get_selection(
        &self,
        preset_handler: &PresetHandler,
        context: FixtureSelectorContext,
    ) -> Result<FixtureSelection, FixtureSelectorError> {
        match self {
            Self::SingleFixture(f) => Ok(vec![*f].into()),
            Self::FixtureRange(begin, end) => Ok((*begin..*end + 1).collect::<Vec<_>>().into()),
            Self::SelectorGroup(s) => s
                .get_selection(preset_handler, context)
                .map(FixtureSelection::from),
            Self::FixtureGroup(id) => {
                let group = preset_handler
                    .get_group(*id)
                    .map_err(FixtureSelectorError::PresetHandlerError)?;
                Ok(group.fixture_selection().clone())
            }
            Self::FixtureIdList(ids) => Ok(ids.clone().into()),
            Self::CurrentFixturesSelected => {
                if let Some(selection) = context.current_fixture_selection {
                    Ok(selection.clone())
                } else {
                    Ok(vec![].into())
                }
            }
            Self::None => Ok(vec![].into()),
        }
    }

    pub fn is_flat(&self) -> bool {
        !matches!(self, Self::CurrentFixturesSelected)
    }
}

impl std::fmt::Display for AtomicFixtureSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CurrentFixturesSelected => write!(f, "~"),
            Self::FixtureGroup(group_id) => write!(f, "Group {}", group_id),
            Self::FixtureIdList(id_list) => write!(f, "{:?}", id_list),
            Self::FixtureRange(from, to) => write!(f, "{} thru {}", from, to),
            Self::SelectorGroup(selector) => write!(f, "({})", selector),
            Self::SingleFixture(id) => write!(f, "{}", id),
            Self::None => write!(f, "None"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FixtureSelector {
    Atomic(AtomicFixtureSelector),
    Additive(AtomicFixtureSelector, Box<FixtureSelector>),
    Subtractive(AtomicFixtureSelector, Box<FixtureSelector>),
    Modulus(AtomicFixtureSelector, u32, bool),
}

impl Default for FixtureSelector {
    fn default() -> Self {
        Self::Atomic(AtomicFixtureSelector::None)
    }
}

impl FixtureSelector {
    pub fn get_selection(
        &self,
        preset_handler: &PresetHandler,
        context: FixtureSelectorContext,
    ) -> Result<FixtureSelection, FixtureSelectorError> {
        match self {
            Self::Atomic(f) => f.get_selection(preset_handler, context),
            Self::Additive(a, b) => {
                let mut fixtures = a.get_selection(preset_handler, context.clone())?;
                fixtures.extend_from(&b.get_selection(preset_handler, context)?);
                Ok(fixtures)
            }
            Self::Subtractive(a, b) => {
                let mut fixtures = a.get_selection(preset_handler, context.clone())?;
                let fixtures_b = b.get_selection(preset_handler, context)?;
                fixtures.subtract(&fixtures_b);
                Ok(fixtures)
            }
            Self::Modulus(fixture_selector, d, invert) => {
                let selection = fixture_selector.get_selection(preset_handler, context)?;
                let mut new_selection = vec![];
                for fixture_id in selection.fixtures() {
                    let fixture_idx = selection.offset_idx(*fixture_id).unwrap();
                    if (fixture_idx as u32 % d == 0) == *invert {
                        continue;
                    }
                    new_selection.push(*fixture_id);
                }
                Ok(new_selection.into())
            }
        }
    }

    pub fn is_flat(&self) -> bool {
        match self {
            Self::Atomic(f) => f.is_flat(),
            Self::Additive(a, b) => a.is_flat() && b.is_flat(),
            Self::Subtractive(a, b) => a.is_flat() && b.is_flat(),
            Self::Modulus(fixture_selector, _, _) => fixture_selector.is_flat(),
        }
    }
}

impl std::fmt::Display for FixtureSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Additive(a, b) => write!(f, "{} + {}", a, b),
            Self::Atomic(a) => write!(f, "{}", a),
            Self::Subtractive(a, b) => write!(f, "{} - {}", a, b),
            Self::Modulus(a, d, invert) => {
                write!(f, "{} %{} {}", a, if *invert { "!" } else { "" }, d)
            }
        }
    }
}
