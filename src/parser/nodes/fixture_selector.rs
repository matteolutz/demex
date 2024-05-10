#[derive(Debug)]
pub enum FixtureSelector {
    SingleFixture(u32),
    FixtureRange(u32, u32),
}

impl FixtureSelector {
    pub fn get_dmx_channels(&self) -> Vec<usize> {
        match self {
            Self::SingleFixture(f) => vec![*f as usize],
            Self::FixtureRange(begin, end) => (*begin as usize..*end as usize).collect(),
        }
    }
}
