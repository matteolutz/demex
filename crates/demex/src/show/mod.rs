use serde::{Deserialize, Serialize};

#[cfg(feature = "ui")]
use ui::DemexShowUiConfig;

use crate::{
    fixture::{
        patch::SerializablePatch, presets::PresetHandler, timing::TimingHandler,
        updatables::UpdatableHandler,
    },
    input::device::DemexInputDeviceConfig,
};

pub mod context;

#[cfg(feature = "ui")]
pub mod ui;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct DemexNoUiShow {
    pub preset_handler: PresetHandler,
    pub updatable_handler: UpdatableHandler,
    pub timing_handler: TimingHandler,
    pub patch: SerializablePatch,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct DemexShow {
    pub preset_handler: PresetHandler,
    pub updatable_handler: UpdatableHandler,
    pub timing_handler: TimingHandler,
    pub input_device_configs: Vec<DemexInputDeviceConfig>,
    pub patch: SerializablePatch,

    #[serde(default)]
    #[cfg(feature = "ui")]
    pub ui_config: DemexShowUiConfig,
}
