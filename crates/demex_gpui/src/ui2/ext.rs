use demex_core::channel3::clamped_value::ClampedValue;
use gpui::{
    App, Bounds, Context, Entity, EventEmitter, Pixels, Point, Size, Subscription, Window, point,
    px, size,
};
use gpui_component::PixelsExt;

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

pub trait BoundsExt {
    /// Map a point in the range 0.0..=1.0 to the bounds' dimensions.
    fn map_point(&self, point: &Point<Pixels>) -> Option<Point<Pixels>>;
    fn map_clamped_point(&self, point: &Point<ClampedValue>) -> Point<Pixels>;

    fn map_clamped_size(&self, size: &Size<ClampedValue>) -> Size<Pixels>;

    fn map_x(&self, x: Pixels) -> Option<Pixels>;
    fn map_y(&self, y: Pixels) -> Option<Pixels>;

    fn map_y_inverted(&self, y: Pixels) -> Option<Pixels>;

    fn unmap_x(&self, x: Pixels) -> Option<Pixels>;
}

impl BoundsExt for Bounds<Pixels> {
    fn map_x(&self, x: Pixels) -> Option<Pixels> {
        let x = x.as_f32();

        if x < 0.0 || x > 1.0 {
            None
        } else {
            Some(self.origin.x + (x * self.size.width))
        }
    }

    fn map_y(&self, y: Pixels) -> Option<Pixels> {
        let y = y.as_f32();

        if y < 0.0 || y > 1.0 {
            None
        } else {
            Some(self.origin.y + (y * self.size.height))
        }
    }

    fn map_y_inverted(&self, y: Pixels) -> Option<Pixels> {
        let y = y.as_f32();

        if y < 0.0 || y > 1.0 {
            None
        } else {
            Some(self.origin.y + ((1.0 - y) * self.size.height))
        }
    }

    fn map_point(&self, relative_point: &Point<Pixels>) -> Option<Point<Pixels>> {
        let x = self.map_x(relative_point.x);
        let y = self.map_y(relative_point.y);

        match (x, y) {
            (Some(x), Some(y)) => Some(point(x, y)),
            _ => None,
        }
    }

    fn map_clamped_point(&self, relative_point: &Point<ClampedValue>) -> Point<Pixels> {
        let x = self.map_x(px(relative_point.x.as_f32())).unwrap();
        let y = self.map_y(px(relative_point.y.as_f32())).unwrap();
        point(x, y)
    }

    fn map_clamped_size(&self, relative_size: &Size<ClampedValue>) -> Size<Pixels> {
        size(
            px(relative_size.width.as_f32()) * self.size.width.as_f32(),
            px(relative_size.height.as_f32()) * self.size.height.as_f32(),
        )
    }

    fn unmap_x(&self, x: Pixels) -> Option<Pixels> {
        if x < self.origin.x || x > self.right() {
            None
        } else {
            Some(px((x - self.origin.x) / self.size.width))
        }
    }
}
