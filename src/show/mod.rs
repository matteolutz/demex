use serde::{Deserialize, Serialize};

use crate::{
    fixture::{
        patch::Patch, presets::PresetHandler, timing::TimingHandler, updatables::UpdatableHandler,
    },
    input::device::DemexInputDeviceConfig,
};

#[derive(Serialize, Deserialize, Default)]
pub struct DemexShow {
    pub preset_handler: PresetHandler,
    pub updatable_handler: UpdatableHandler,
    pub timing_handler: TimingHandler,
    pub input_device_configs: Vec<DemexInputDeviceConfig>,
    pub patch: Patch,
}
