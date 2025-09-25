use crate::fixture::presets::preset::FixturePresetTarget;

impl FixturePresetTarget {
    pub fn get_color(&self) -> ecolor::Color32 {
        match self {
            Self::AllSelected => ecolor::Color32::LIGHT_GREEN,
            Self::SomeSelected => ecolor::Color32::LIGHT_BLUE,
            Self::None => ecolor::Color32::LIGHT_RED,
        }
    }

    pub fn get_short_name(&self) -> &str {
        match self {
            Self::AllSelected => "A",
            Self::SomeSelected => "S",
            Self::None => "N",
        }
    }
}
