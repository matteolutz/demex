use gpui::{Context, Entity, EventEmitter, Subscription};

pub trait GpuiContextExtension<T> {
    /// Subscribe to an event type from another entity
    fn subscribe_with<T2, T3, Evt>(
        &mut self,
        entity: &Entity<T2>,
        entity2: Entity<T3>,
        on_event: impl FnMut(&mut T, Entity<T2>, &Evt, Entity<T3>, &mut Context<T>) + 'static,
    ) -> Subscription
    where
        T: 'static,
        T2: 'static + EventEmitter<Evt>,
        T3: 'static,
        Evt: 'static;
}

impl<'a, T: 'static> GpuiContextExtension<T> for Context<'a, T> {
    fn subscribe_with<T2, T3, Evt>(
        &mut self,
        entity: &Entity<T2>,
        entity2: Entity<T3>,
        mut on_event: impl FnMut(&mut T, Entity<T2>, &Evt, Entity<T3>, &mut Context<T>) + 'static,
    ) -> Subscription
    where
        T: 'static,
        T2: 'static + EventEmitter<Evt>,
        T3: 'static,
        Evt: 'static,
    {
        let entity2 = entity2.downgrade();

        self.subscribe(entity, move |this, entity, event, cx| {
            let Some(entity2) = entity2.upgrade() else {
                return;
            };

            on_event(this, entity, event, entity2, cx)
        })
    }
}
