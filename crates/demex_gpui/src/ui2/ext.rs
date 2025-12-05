use gpui::{App, Bounds, Context, Entity, EventEmitter, Pixels, Subscription, Window};

pub trait GpuiContextExtension<T> {
    /// Arranges so that [`Context::notify`] will be called for the current context
    /// whenever [`Context::notify`] is called with the given entity.
    fn observe_and_notify<W>(&mut self, entity: &Entity<W>) -> Subscription
    where
        W: 'static;

    /// Subscribe to an event type from another entity also accepting a second entity
    /// that will be passed to the event handler.
    fn subscribe_with_entity<T2, T3, Evt>(
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

    /// Convenience method for producing view state in a `Canvas` draw method.
    /// See `listener` for more details.
    fn draw_canvas<C>(
        &self,
        f: impl 'static + FnOnce(&mut T, Bounds<Pixels>, C, &mut Window, &mut Context<T>),
    ) -> impl 'static + FnOnce(Bounds<Pixels>, C, &mut Window, &mut App);
}

impl<'a, T: 'static> GpuiContextExtension<T> for Context<'a, T> {
    fn observe_and_notify<W>(&mut self, entity: &Entity<W>) -> Subscription
    where
        W: 'static,
    {
        self.observe(entity, |_, _, cx| cx.notify())
    }

    fn subscribe_with_entity<T2, T3, Evt>(
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

    fn draw_canvas<C>(
        &self,
        f: impl 'static + FnOnce(&mut T, Bounds<Pixels>, C, &mut Window, &mut Context<T>),
    ) -> impl 'static + FnOnce(Bounds<Pixels>, C, &mut Window, &mut App) {
        let view = self.entity().downgrade();
        move |bounds: Bounds<Pixels>, c: C, window: &mut Window, cx: &mut App| {
            view.update(cx, |view, cx| f(view, bounds, c, window, cx))
                .ok();
        }
    }
}
