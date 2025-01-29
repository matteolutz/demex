#[derive(Debug)]
pub enum DemexUiError {
    RuntimeError(String),
}

impl std::error::Error for DemexUiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for DemexUiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RuntimeError(error) => write!(f, "DemexUiError: Runtime Error: {}", error),
        }
    }
}
