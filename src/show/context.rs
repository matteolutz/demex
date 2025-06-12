use std::sync::Arc;

use parking_lot::RwLock;

use crate::{
    fixture::{
        handler::FixtureHandler,
        patch::{Patch, SerializablePatch},
        presets::PresetHandler,
        timing::TimingHandler,
        updatables::UpdatableHandler,
    },
    headless::id::DemexProtoDeviceId,
};

use super::DemexNoUiShow;

#[derive(Clone, Debug)]
pub struct ShowContext {
    pub fixture_handler: Arc<RwLock<FixtureHandler>>,
    pub preset_handler: Arc<RwLock<PresetHandler>>,
    pub updatable_handler: Arc<RwLock<UpdatableHandler>>,
    pub timing_handler: Arc<RwLock<TimingHandler>>,
    pub patch: Arc<RwLock<Patch>>,
}

impl ShowContext {
    pub fn update_from(&mut self, show: DemexNoUiShow, own_device_id: DemexProtoDeviceId) {
        let patch = show
            .patch
            .into_patch(self.patch.read().fixture_types().to_vec());
        let (fixtures, outputs) = patch.into_fixures_and_outputs(own_device_id);

        *self.fixture_handler.write() = FixtureHandler::new(fixtures, outputs).unwrap();
        *self.preset_handler.write() = show.preset_handler;
        *self.updatable_handler.write() = show.updatable_handler;
        *self.timing_handler.write() = show.timing_handler;
        *self.patch.write() = patch;
    }

    pub fn new(
        fixture_types: Vec<gdtf::fixture_type::FixtureType>,
        patch: SerializablePatch,
        preset_handler: PresetHandler,
        updatable_handler: UpdatableHandler,
        timing_handler: TimingHandler,
        own_device_id: DemexProtoDeviceId,
    ) -> Self {
        let patch = Arc::new(RwLock::new(patch.into_patch(fixture_types)));
        let (fixtures, outputs) = patch.read().into_fixures_and_outputs(own_device_id);

        let fixture_handler =
            Arc::new(RwLock::new(FixtureHandler::new(fixtures, outputs).unwrap()));

        let preset_handler = Arc::new(RwLock::new(preset_handler));
        let updatable_handler = Arc::new(RwLock::new(updatable_handler));
        let timing_handler = Arc::new(RwLock::new(timing_handler));

        Self {
            fixture_handler,
            preset_handler,
            updatable_handler,
            timing_handler,
            patch,
        }
    }
}
