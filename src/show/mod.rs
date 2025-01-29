use serde::{Deserialize, Serialize};

use crate::fixture::presets::PresetHandler;

#[derive(Serialize, Deserialize)]
pub struct DemexShow {
    pub preset_handler: PresetHandler,
}
