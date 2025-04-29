#[derive(Debug)]
pub enum DemexGdtfError {
    UnnamedFeature,
    UnnamedFeatureGroup,
    UnnamedAttribute,
}

impl std::fmt::Display for DemexGdtfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnnamedFeature => write!(f, "A feature in a fixture type definition is unnamed"),
            Self::UnnamedFeatureGroup => {
                write!(f, "A feature group in a fixture type definition is unnamed")
            }
            Self::UnnamedAttribute => {
                write!(f, "An attribute in a fixture type definition is unnamed")
            }
        }
    }
}

impl std::error::Error for DemexGdtfError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
