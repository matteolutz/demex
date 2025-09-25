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
};

pub mod component;
pub mod error;
pub mod threads;

pub struct DemexEngine {
    components: HashMap<TypeId, Arc<Mutex<dyn Any + Send + Sync>>>,
}

impl DemexEngine {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
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
            .expect("component not registered");
        ComponentHandle::new(component.clone())
    }

    pub fn start(&mut self) {}

    #[inline]
    pub fn fixture_handler(&self) -> ComponentHandle<FixtureHandler> {
        self.component()
    }
}
