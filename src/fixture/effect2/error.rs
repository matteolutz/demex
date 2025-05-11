#[derive(Debug)]
pub enum Effect2Error {
    WaveControlPointTypeMismatch,
}

impl std::error::Error for Effect2Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for Effect2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WaveControlPointTypeMismatch => write!(f, "Wave control point type mismatch"),
        }
    }
}
