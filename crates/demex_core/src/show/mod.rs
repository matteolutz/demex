use serde::{Deserialize, Serialize};

use crate::{
    input::device::DemexInputDeviceConfig, patch::SerializablePatch, presets::PresetHandler,
    timing::TimingHandler, updatables::UpdatableHandler,
};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct DemexShow {
    pub preset_handler: PresetHandler,
    pub updatable_handler: UpdatableHandler,
    pub timing_handler: TimingHandler,
    pub input_device_configs: Vec<DemexInputDeviceConfig>,
    pub patch: SerializablePatch,
}
