#[derive(Debug)]
pub enum FixtureSelector {
    SingleFixture(u32),
    FixtureRange(u32, u32),
}

impl FixtureSelector {
    pub fn get_fixtures(&self) -> Vec<u32> {
        match self {
            Self::SingleFixture(f) => vec![*f],
            Self::FixtureRange(begin, end) => (*begin..*end + 1).collect(),
        }
    }
}
