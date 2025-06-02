use strum::EnumIter;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, EnumIter)]
pub enum ClockMode {
    #[default]
    Timecode,

    LocalTime,
    Utc,
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
