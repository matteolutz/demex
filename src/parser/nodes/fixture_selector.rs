use serde::{Deserialize, Serialize};

use crate::fixture::presets::{error::PresetHandlerError, PresetHandler};

#[derive(Debug, Clone)]
pub struct FixtureSelectorContext<'a> {
    current_fixture_selector: &'a Option<FixtureSelector>,
}

impl<'a> FixtureSelectorContext<'a> {
    pub fn new(current_fixture_selector: &'a Option<FixtureSelector>) -> Self {
        Self {
            current_fixture_selector,
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
    pub fn get_fixtures(
        &self,
        preset_handler: &PresetHandler,
        context: FixtureSelectorContext,
    ) -> Result<Vec<u32>, FixtureSelectorError> {
        match self {
            Self::SingleFixture(f) => Ok(vec![*f]),
            Self::FixtureRange(begin, end) => Ok((*begin..*end + 1).collect()),
            Self::SelectorGroup(s) => s.get_fixtures(preset_handler, context),
            Self::FixtureGroup(id) => {
                let group = preset_handler
                    .get_group(*id)
                    .map_err(FixtureSelectorError::PresetHandlerError)?;
                group.get_fixtures(preset_handler, context)
            }
            Self::FixtureIdList(ids) => Ok(ids.clone()),
            Self::CurrentFixturesSelected => {
                if let Some(selector) = context.current_fixture_selector {
                    selector.get_fixtures(preset_handler, context)
                } else {
                    Ok(vec![])
                }
            }
            Self::None => Ok(vec![]),
        }
    }

    pub fn flatten(
        &self,
        preset_handler: &PresetHandler,
        context: FixtureSelectorContext,
    ) -> Result<Self, FixtureSelectorError> {
        match self {
            Self::CurrentFixturesSelected => {
                let fixtures = self
                    .get_fixtures(preset_handler, context)
                    .map_err(|e| FixtureSelectorError::FailedToFlatten(Box::new(e)))?;
                Ok(Self::FixtureIdList(fixtures))
            }
            _ => Ok(self.clone()),
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
    pub fn get_fixtures(
        &self,
        preset_handler: &PresetHandler,
        context: FixtureSelectorContext,
    ) -> Result<Vec<u32>, FixtureSelectorError> {
        match self {
            Self::Atomic(f) => f.get_fixtures(preset_handler, context),
            Self::Additive(a, b) => {
                let mut fixtures = a.get_fixtures(preset_handler, context.clone())?;
                fixtures.extend(b.get_fixtures(preset_handler, context)?);
                Ok(fixtures)
            }
            Self::Subtractive(a, b) => {
                let mut fixtures = a.get_fixtures(preset_handler, context.clone())?;
                let fixtures_b = b.get_fixtures(preset_handler, context)?;
                fixtures.retain(|f| !fixtures_b.contains(f));
                Ok(fixtures)
            }
            Self::Modulus(fixture_selector, d, invert) => {
                let mut fixtures = fixture_selector.get_fixtures(preset_handler, context)?;
                fixtures = fixtures
                    .iter()
                    .enumerate()
                    .filter(|&(idx, _)| (idx as u32 % d == 0) != *invert)
                    .map(|(_, val)| *val)
                    .collect::<Vec<u32>>();
                Ok(fixtures)
            }
        }
    }

    pub fn flatten(
        &self,
        preset_handler: &PresetHandler,
        context: FixtureSelectorContext,
    ) -> Result<Self, FixtureSelectorError> {
        match self {
            Self::Atomic(f) => f.flatten(preset_handler, context).map(Self::Atomic),
            Self::Additive(a, b) => {
                let a = a.flatten(preset_handler, context.clone())?;
                let b = b.flatten(preset_handler, context)?;
                Ok(Self::Additive(a, Box::new(b)))
            }
            Self::Subtractive(a, b) => {
                let a = a.flatten(preset_handler, context.clone())?;
                let b = b.flatten(preset_handler, context)?;
                Ok(Self::Subtractive(a, Box::new(b)))
            }
            Self::Modulus(fixture_selector, d, invert) => {
                let fixture_selector = fixture_selector.flatten(preset_handler, context)?;
                Ok(Self::Modulus(fixture_selector, *d, *invert))
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
