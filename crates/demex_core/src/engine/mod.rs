/*
 *
 * This file has been modified from its original version.
 * Original: https://github.com/BaukeWestendorp/radiant
 * License: Apache 2.0 - https://github.com/BaukeWestendorp/radiant/blob/main/LICENCE
 *
 */

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use parking_lot::Mutex;

use crate::{
    engine::component::{Component, ComponentHandle},
    fixture::handler::FixtureHandler,
    input::event::handler::DemexInputDeviceEventHandler,
    patch::Patch,
    presets::PresetHandler,
    timing::TimingHandler,
    updatables::UpdatableHandler,
    utils::thread::DemexThreadStatsHandler,
};

pub mod component;
pub mod error;
pub mod threads;

pub struct DemexEngine {
    components: HashMap<TypeId, Arc<Mutex<dyn Any + Send + Sync>>>,
    stats: ComponentHandle<DemexThreadStatsHandler>,
}

impl DemexEngine {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            stats: ComponentHandle::create_default(),
        }
    }
}

impl DemexEngine {
    pub fn register_component<T>(&mut self, component: T)
    where
        T: Component + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        self.components
            .insert(type_id, Arc::new(Mutex::new(component)));
    }

    pub fn component<T: Component + 'static>(&self) -> ComponentHandle<T> {
        let type_id = TypeId::of::<T>();
        let component = self
            .components
            .get(&type_id)
            .expect(format!("Component {:?} not registered", type_id).as_str());
        ComponentHandle::new(component.clone())
    }

    pub fn start(&mut self) {
        threads::update::start_demex_update_thread(
            self.stats(),
            self.fixture_handler(),
            self.preset_handler(),
            self.updatable_handler(),
            self.timing_handler(),
            self.patch(),
            self.input_device_event_handler(),
        );

        threads::output::start_demex_output_thread(
            self.stats(),
            self.fixture_handler(),
            self.preset_handler(),
            self.timing_handler(),
            self.patch(),
        );

        threads::debug::start_demex_debug_thread(self.stats());
    }

    #[inline]
    pub fn fixture_handler(&self) -> ComponentHandle<FixtureHandler> {
        self.component()
    }

    #[inline]
    pub fn preset_handler(&self) -> ComponentHandle<PresetHandler> {
        self.component()
    }

    #[inline]
    pub fn updatable_handler(&self) -> ComponentHandle<UpdatableHandler> {
        self.component()
    }

    #[inline]
    pub fn timing_handler(&self) -> ComponentHandle<TimingHandler> {
        self.component()
    }

    #[inline]
    pub fn patch(&self) -> ComponentHandle<Patch> {
        self.component()
    }

    #[inline]
    pub fn input_device_event_handler(&self) -> ComponentHandle<DemexInputDeviceEventHandler> {
        self.component()
    }

    #[inline]
    pub fn stats(&self) -> ComponentHandle<DemexThreadStatsHandler> {
        self.stats.clone()
    }
}
