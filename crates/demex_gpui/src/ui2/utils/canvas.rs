use gpui::{Bounds, Canvas, Entity, Pixels, canvas};

/// Convenience method for creating a canvas that writes its bounds to the given entity
/// and notifies it.
pub fn bounds(bounds: &Entity<Option<Bounds<Pixels>>>) -> Canvas<()> {
    let bounds = bounds.clone();

    canvas(
        move |canvas_bounds, _, cx| {
            bounds.update(cx, |bounds, cx| {
                *bounds = Some(canvas_bounds);
                cx.notify();
            });
        },
        |_, _, _, _| {},
    )
}
