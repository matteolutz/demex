#[derive(Debug)]
pub enum MidiError {
    MidirError(Box<dyn std::error::Error>),
}

impl std::fmt::Display for MidiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MidiError::MidirError(err) => write!(f, "Midir error: {}", err),
        }
    }
}

impl std::error::Error for MidiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
