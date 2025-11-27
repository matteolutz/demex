/*
 *
 * This file has been modified from its original version.
 * Original: https://github.com/BaukeWestendorp/radiant
 * License: Apache 2.0 - https://github.com/BaukeWestendorp/radiant/blob/main/LICENCE
 *
 */

use std::{any::Any, marker::PhantomData, sync::Arc};

use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

pub trait Component: Send + Sync + Default + 'static {}

pub struct ComponentHandle<T: Component>(Arc<RwLock<dyn Any + Send + Sync>>, PhantomData<T>);

impl<T: Component> ComponentHandle<T> {
    pub(crate) fn create(component: T) -> Self {
        Self::new(Arc::new(RwLock::new(component)))
    }

    pub(crate) fn create_default() -> Self {
        Self::create(T::default())
    }

    pub(crate) fn new(component: Arc<RwLock<dyn Any + Send + Sync>>) -> Self {
        Self(component, PhantomData::default())
    }

    pub fn read<R, F: FnOnce(&T) -> R>(&self, f: F) -> R {
        let guard = self.lock_read();
        f(&guard)
    }

    // Only allow core_crate to write
    pub(crate) fn write<R, F: FnOnce(&mut T) -> R>(&mut self, f: F) -> R {
        let mut guard = self.lock_write();
        f(&mut guard)
    }

    pub(crate) fn lock_read(&self) -> MappedRwLockReadGuard<'_, T> {
        let guard = self.0.read();
        RwLockReadGuard::map(guard, |any: &(dyn Any + Send + Sync + 'static)| {
            any.downcast_ref::<T>().unwrap()
        })
    }

    pub(crate) fn lock_write(&self) -> MappedRwLockWriteGuard<'_, T> {
        let guard = self.0.write();
        RwLockWriteGuard::map(guard, |any: &mut (dyn Any + Send + Sync + 'static)| {
            any.downcast_mut::<T>().unwrap()
        })
    }
}

impl<T: Component> Clone for ComponentHandle<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}
