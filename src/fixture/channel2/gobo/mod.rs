use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GoboMacro {
    name: String,
    image: Option<PathBuf>,
}

impl GoboMacro {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn image(&self) -> Option<&PathBuf> {
        self.image.as_ref()
    }
}
