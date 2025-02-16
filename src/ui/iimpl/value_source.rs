use crate::fixture::value_source::FixtureChannelValueSource;

impl FixtureChannelValueSource {
    pub fn get_color(&self) -> egui::Color32 {
        match &self {
            Self::Programmer => egui::Color32::YELLOW,
            Self::Fader { .. } => egui::Color32::LIGHT_BLUE,
            Self::Executor { .. } => egui::Color32::LIGHT_GREEN,
        }
    }
}
