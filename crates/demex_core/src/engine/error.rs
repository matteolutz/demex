#[derive(Debug)]
pub enum DemexEngineError {
    SerdeJsonError(serde_json::Error),
}

impl std::fmt::Display for DemexEngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SerdeJsonError(error) => write!(f, "Serde JSON error: {}", error),
        }
    }
}

impl std::error::Error for DemexEngineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
