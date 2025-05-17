use crate::fixture::value_source::FixtureChannelValueSource;

impl FixtureChannelValueSource {
    pub fn get_color(&self) -> ecolor::Color32 {
        match &self {
            Self::Programmer => ecolor::Color32::YELLOW,
            Self::Fader { .. } => ecolor::Color32::LIGHT_BLUE,
            Self::Executor { .. } => ecolor::Color32::LIGHT_GREEN,
        }
    }
}
