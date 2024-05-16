#[derive(Debug, Clone)]
pub enum AtomicFixtureSelector {
    SingleFixture(u32),
    FixtureRange(u32, u32),
    SelectorGroup(Box<FixtureSelector>),
}

impl AtomicFixtureSelector {
    pub fn get_fixtures(&self) -> Vec<u32> {
        match self {
            Self::SingleFixture(f) => vec![*f],
            Self::FixtureRange(begin, end) => (*begin..*end + 1).collect(),
            Self::SelectorGroup(s) => s.get_fixtures(),
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
    pub fn get_fixtures(&self) -> Vec<u32> {
        match self {
            Self::Atomic(f) => f.get_fixtures(),
            Self::Additive(a, b) => {
                let mut fixtures = a.get_fixtures();
                fixtures.extend(b.get_fixtures());
                fixtures
            }
            Self::Subtractive(a, b) => {
                let mut fixtures = a.get_fixtures();
                fixtures.retain(|f| !b.get_fixtures().contains(f));
                fixtures
            }
            Self::Modulus(fixture_selector, d, invert) => {
                let mut fixtures = fixture_selector.get_fixtures();
                fixtures.retain(|f| (f % d == 0) != *invert);
                fixtures
            }
        }
    }
}
