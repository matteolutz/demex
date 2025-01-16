use serde::{Deserialize, Serialize};

use crate::fixture::presets::PresetHandler;

#[derive(Serialize, Deserialize)]
pub struct DemexShow {
    preset_handler: PresetHandler,
}

impl DemexShow {
    pub fn preset_handler(&self) -> &PresetHandler {
        &self.preset_handler
    }
}
