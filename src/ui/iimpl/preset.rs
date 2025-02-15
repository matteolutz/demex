use crate::fixture::presets::preset::FixturePresetTarget;

impl FixturePresetTarget {
    pub fn get_color(&self) -> egui::Color32 {
        match self {
            Self::AllSelected => egui::Color32::LIGHT_GREEN,
            Self::SomeSelected => egui::Color32::LIGHT_BLUE,
            Self::None => egui::Color32::LIGHT_RED,
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
