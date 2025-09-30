/*
 *
 * This file has been modified from its original version.
 * Original: https://github.com/BaukeWestendorp/radiant
 * License: Apache 2.0 - https://github.com/BaukeWestendorp/radiant/blob/main/LICENCE
 *
 */

use std::{any::Any, marker::PhantomData, sync::Arc};

use parking_lot::{
    Mutex, RawMutex,
    lock_api::{MappedMutexGuard, MutexGuard},
};

pub trait Component: Send + Sync + Default + 'static {}

pub struct ComponentHandle<T: Component>(Arc<Mutex<dyn Any + Send + Sync>>, PhantomData<T>);

impl<T: Component> ComponentHandle<T> {
    pub(crate) fn create(component: T) -> Self {
        Self::new(Arc::new(Mutex::new(component)))
    }

    pub(crate) fn create_default() -> Self {
        Self::create(T::default())
    }

    pub(crate) fn new(component: Arc<Mutex<dyn Any + Send + Sync>>) -> Self {
        Self(component, PhantomData::default())
    }

    pub fn read<R, F: FnOnce(&T) -> R>(&self, f: F) -> R {
        let guard = self.lock();
        f(&guard)
    }

    pub fn write<R, F: FnOnce(&mut T) -> R>(&mut self, f: F) -> R {
        let mut guard = self.lock();
        f(&mut guard)
    }

    pub fn lock(&self) -> MappedMutexGuard<'_, RawMutex, T> {
        let guard = self.0.lock();
        MutexGuard::map(guard, |any| any.downcast_mut::<T>().unwrap())
    }
}

impl<T: Component> Clone for ComponentHandle<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}
