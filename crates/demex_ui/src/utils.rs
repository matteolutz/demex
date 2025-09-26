use gpui::prelude::*;
use gpui::{App, Bounds, Canvas, Div, Entity, FontWeight, Pixels, Point, canvas, div};

use crate::theme::{ActiveTheme, InteractiveColor};

/// Stack elements on top of each other.
pub fn z_stack(children: impl IntoIterator<Item = impl IntoElement>) -> Div {
    let children = children
        .into_iter()
        .map(|child| div().size_full().child(child).absolute());
    div().relative().children(children)
}

/// Create a listener that gets the bounds of the element.
pub fn bounds_updater<V: 'static>(
    entity: Entity<V>,
    f: impl FnOnce(&mut V, Bounds<Pixels>, &mut Context<V>) + 'static,
) -> Canvas<()> {
    let entity = entity.clone();
    canvas(
        move |bounds, _window, cx| entity.update(cx, |view, cx| f(view, bounds, cx)),
        |_, _, _, _| {},
    )
    .size_full()
}

pub enum SnapModeFunction {
    Floor,
    Round,
    Ceil,
}

impl SnapModeFunction {
    pub fn apply(&self, value: f32) -> f32 {
        match self {
            SnapModeFunction::Floor => value.floor(),
            SnapModeFunction::Round => value.round(),
            SnapModeFunction::Ceil => value.ceil(),
        }
    }
}

/// Snap a point to the nearest multiple of the given threshold.
pub fn snap_point(
    mut point: Point<Pixels>,
    threshold: Pixels,
    mode: SnapModeFunction,
) -> Point<Pixels> {
    point.x = mode.apply(point.x / threshold) * threshold;
    point.y = mode.apply(point.y / threshold) * threshold;
    point
}

/// Snap a point to the nearest multiple of the given threshold.
pub fn snap_point_2(
    mut point: Point<Pixels>,
    threshold: Point<Pixels>,
    mode: SnapModeFunction,
) -> Point<Pixels> {
    point.x = mode.apply(point.x / threshold.x) * threshold.x;
    point.y = mode.apply(point.y / threshold.y) * threshold.y;
    point
}

/// Creates a placeholder TODO element.
pub fn todo(cx: &App) -> Div {
    div()
        .size_full()
        .flex()
        .justify_center()
        .items_center()
        .bg(cx.theme().red.with_opacity(0.1))
        .text_color(cx.theme().red)
        .font_weight(FontWeight::BOLD)
        .border_1()
        .border_color(cx.theme().red)
        .child("TODO")
}
