use gdtf::fixture_type::FixtureType;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ui")]
use ui::DemexShowUiConfig;

use crate::{
    engine::DemexEngine,
    fixture::handler::FixtureHandler,
    input::{device::DemexInputDeviceConfig, event::handler::DemexInputDeviceEventHandler},
    patch::SerializablePatch,
    presets::PresetHandler,
    timing::TimingHandler,
    updatables::UpdatableHandler,
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

impl DemexShow {
    pub fn register(self, global_fixture_types: Vec<FixtureType>, engine: &mut DemexEngine) {
        let patch = self.patch.into_patch(global_fixture_types);
        let (fixtures, outputs) =
            patch.into_fixures_and_outputs(demex_headless::id::DemexProtoDeviceId::Controller);

        engine.register_component(FixtureHandler::new(fixtures, outputs).unwrap());
        engine.register_component(self.preset_handler);
        engine.register_component(self.updatable_handler);
        engine.register_component(self.timing_handler);
        engine.register_component(patch.clone());
        engine.register_component(DemexInputDeviceEventHandler::new());
    }
}
