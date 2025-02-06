use serde::{Deserialize, Serialize};

use crate::fixture::{presets::PresetHandler, updatables::UpdatableHandler};

#[derive(Serialize, Deserialize)]
pub struct DemexShow {
    pub preset_handler: PresetHandler,
    pub updatable_handler: UpdatableHandler,
}
