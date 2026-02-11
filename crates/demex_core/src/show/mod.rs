use serde::{Deserialize, Serialize};

use crate::{
    input::{DemexInputDeviceHandler, device::DemexInputDeviceConfig},
    master::{MasterConfig, MasterHandler},
    patch::{Patch, SerializablePatch},
    pool::{Pool, PoolType},
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

    #[serde(default)]
    pub master_config: MasterConfig,

    pub patch: SerializablePatch,
}

#[derive(Debug, Copy, Clone)]
pub struct DemexShowRef<'a> {
    pub preset_handler: &'a PresetHandler,
    pub updatable_handler: &'a UpdatableHandler,
    pub timing_handler: &'a TimingHandler,
    pub input_device_handler: &'a DemexInputDeviceHandler,
    pub master_handler: &'a MasterHandler,
    pub patch: &'a Patch,
}

impl<'a> DemexShowRef<'a> {
    pub fn clone_into_show(self) -> DemexShow {
        DemexShow {
            preset_handler: self.preset_handler.clone(),
            updatable_handler: self.updatable_handler.clone(),
            timing_handler: self.timing_handler.clone(),
            input_device_configs: self
                .input_device_handler
                .device_configs()
                .cloned()
                .collect(),
            master_config: self.master_handler.into(),
            patch: self.patch.patch.clone(),
        }
    }

    pub fn get_pool(&self, pool_type: PoolType) -> &dyn Pool {
        match pool_type {
            PoolType::Executor => self.updatable_handler,
            PoolType::Preset(_)
            | PoolType::Sequence
            | PoolType::SequenceCue(_)
            | PoolType::Group
            | PoolType::Macro => self.preset_handler,
        }
    }
}
