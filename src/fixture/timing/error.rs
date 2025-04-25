#[derive(Debug)]
pub enum TimingHandlerError {
    SpeedMasterValueNotFound(u32),
    TimecodeNotFound(u32),
}

impl std::fmt::Display for TimingHandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SpeedMasterValueNotFound(id) => {
                write!(f, "SpeedMasterValue with id {} not found", id)
            }
            Self::TimecodeNotFound(id) => {
                write!(f, "Timecode with id {} not found", id)
            }
        }
    }
}

impl std::error::Error for TimingHandlerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
