use serde::{Deserialize, Serialize};

use crate::{
    input::device::DemexInputDeviceConfig,
    patch::{Patch, SerializablePatch},
    presets::PresetHandler,
    timing::TimingHandler,
    updatables::UpdatableHandler,
};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct DemexShow {
    pub preset_handler: PresetHandler,
    pub updatable_handler: UpdatableHandler,
    pub timing_handler: TimingHandler,
    pub input_device_configs: Vec<DemexInputDeviceConfig>,
    pub patch: SerializablePatch,
}

#[derive(Debug, Copy, Clone)]
pub struct DemexShowRef<'a> {
    pub preset_handler: &'a PresetHandler,
    pub updatable_handler: &'a UpdatableHandler,
    pub timing_handler: &'a TimingHandler,
    pub input_device_configs: &'a Vec<DemexInputDeviceConfig>,
    pub patch: &'a Patch,
}

impl<'a> DemexShowRef<'a> {
    pub fn clone_into_show(self) -> DemexShow {
        DemexShow {
            preset_handler: self.preset_handler.clone(),
            updatable_handler: self.updatable_handler.clone(),
            timing_handler: self.timing_handler.clone(),
            input_device_configs: self.input_device_configs.clone(),
            patch: SerializablePatch::from_patch(self.patch),
        }
    }
}
