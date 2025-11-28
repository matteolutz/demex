use strum::EnumIter;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, EnumIter)]
pub enum ClockMode {
    #[default]
    Timecode,

    LocalTime,
    Utc,
}

impl ClockMode {
    pub fn color(&self) -> egui::Color32 {
        match self {
            ClockMode::Timecode => egui::Color32::LIGHT_GREEN,
            ClockMode::LocalTime | ClockMode::Utc => egui::Color32::ORANGE,
        }
    }
}

impl std::fmt::Display for ClockMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClockMode::Timecode => write!(f, "Timecode"),
            ClockMode::LocalTime => write!(f, "Local Time"),
            ClockMode::Utc => write!(f, "UTC"),
        }
    }
}
