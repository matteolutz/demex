use crate::fixture::presets::{error::PresetHandlerError, PresetHandler};

#[derive(Debug)]
pub enum FixtureSelectorError {
    PresetHandlerError(PresetHandlerError),
}

impl std::fmt::Display for FixtureSelectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::PresetHandlerError(e) => write!(f, "PresetHandlerError: {}", e),
        }
    }
}

impl std::error::Error for FixtureSelectorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[derive(Debug, Clone)]
pub enum AtomicFixtureSelector {
    SingleFixture(u32),
    FixtureRange(u32, u32),
    FixtureGroup(u32),
    SelectorGroup(Box<FixtureSelector>),
}

impl AtomicFixtureSelector {
    pub fn get_fixtures(
        &self,
        preset_handler: &PresetHandler,
    ) -> Result<Vec<u32>, FixtureSelectorError> {
        match self {
            Self::SingleFixture(f) => Ok(vec![*f]),
            Self::FixtureRange(begin, end) => Ok((*begin..*end + 1).collect()),
            Self::SelectorGroup(s) => s.get_fixtures(preset_handler),
            Self::FixtureGroup(id) => {
                let group = preset_handler
                    .get_group(*id)
                    .map_err(FixtureSelectorError::PresetHandlerError)?;
                group.get_fixtures(preset_handler)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum FixtureSelector {
    Atomic(AtomicFixtureSelector),
    Additive(AtomicFixtureSelector, Box<FixtureSelector>),
    Subtractive(AtomicFixtureSelector, Box<FixtureSelector>),
    Modulus(AtomicFixtureSelector, u32, bool),
}

impl FixtureSelector {
    pub fn get_fixtures(
        &self,
        preset_handler: &PresetHandler,
    ) -> Result<Vec<u32>, FixtureSelectorError> {
        match self {
            Self::Atomic(f) => f.get_fixtures(preset_handler),
            Self::Additive(a, b) => {
                let mut fixtures = a.get_fixtures(preset_handler)?;
                fixtures.extend(b.get_fixtures(preset_handler)?);
                Ok(fixtures)
            }
            Self::Subtractive(a, b) => {
                let mut fixtures = a.get_fixtures(preset_handler)?;
                let fixtures_b = b.get_fixtures(preset_handler)?;
                fixtures.retain(|f| !fixtures_b.contains(f));
                Ok(fixtures)
            }
            Self::Modulus(fixture_selector, d, invert) => {
                let mut fixtures = fixture_selector.get_fixtures(preset_handler)?;
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
}
