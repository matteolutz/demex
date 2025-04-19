use serde::{Deserialize, Serialize};
use ui::DemexShowUiConfig;

use crate::{
    fixture::{
        patch::SerializablePatch, presets::PresetHandler, timing::TimingHandler,
        updatables::UpdatableHandler,
    },
    input::device::DemexInputDeviceConfig,
};

pub mod ui;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct DemexShow {
    pub preset_handler: PresetHandler,
    pub updatable_handler: UpdatableHandler,
    pub timing_handler: TimingHandler,
    pub input_device_configs: Vec<DemexInputDeviceConfig>,
    pub patch: SerializablePatch,

    #[serde(default)]
    pub ui_config: DemexShowUiConfig,
}
