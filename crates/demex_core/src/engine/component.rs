/*
 *
 * This file has been modified from its original version.
 * Original: https://github.com/BaukeWestendorp/radiant
 * License: Apache 2.0 - https://github.com/BaukeWestendorp/radiant/blob/main/LICENCE
 *
 */

use std::{any::Any, marker::PhantomData, sync::Arc};

use parking_lot::Mutex;

pub trait Component: Send + Sync + Default + 'static {}

pub struct ComponentHandle<T: Component>(Arc<Mutex<dyn Any + Send + Sync>>, PhantomData<T>);

impl<T: Component> ComponentHandle<T> {
    pub(crate) fn new(component: Arc<Mutex<dyn Any + Send + Sync>>) -> Self {
        Self(component, PhantomData::default())
    }

    pub fn read<R, F: FnOnce(&T) -> R>(&self, f: F) -> R {
        let guard = self.0.lock();
        let component = guard.downcast_ref::<T>().expect("Component type mismatch");
        f(component)
    }

    pub(crate) fn write<R, F: FnOnce(&mut T) -> R>(&mut self, f: F) -> R {
        let mut guard = self.0.lock();
        let component = guard.downcast_mut::<T>().expect("Component type mismatch");
        f(component)
    }

    pub fn mutex(&self) -> &Arc<Mutex<dyn Any + Send + Sync>> {
        &self.0
    }

    pub fn mutex_mut(&mut self) -> &mut Arc<Mutex<dyn Any + Send + Sync>> {
        &mut self.0
    }
}

impl<T: Component> Clone for ComponentHandle<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}
