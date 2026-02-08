use demex_core::channel3::clamped_value::ClampedValue;
use gpui::{
    App, Bounds, Context, Corners, Edges, Entity, InteractiveElement, IntoElement, MouseButton,
    PaintQuad, ParentElement, Pixels, Point, RenderOnce, Styled, Window, canvas, div, fill,
    outline, point, px, size, transparent_black,
};
use gpui_component::{ActiveTheme, black, white};

use crate::ui2::ext::BoundsExt;

pub type PanTiltEditorValue = [ClampedValue; 2];

pub struct PanTiltEditorState {
    value: PanTiltEditorValue,

    canvas_bounds: Bounds<Pixels>,
    mouse_down_pos: Option<Point<Pixels>>,
}

impl PanTiltEditorState {
    pub fn new(value: PanTiltEditorValue, _cx: &mut Context<Self>) -> Self {
        Self {
            value,

            canvas_bounds: Bounds::default(),
            mouse_down_pos: None,
        }
    }

    pub fn value(&self) -> PanTiltEditorValue {
        self.value
    }

    pub fn set_value(&mut self, value: PanTiltEditorValue, cx: &mut Context<Self>) {
        self.value = value;
        cx.notify();
    }

    fn mouse_down(&mut self, pos: Point<Pixels>, cx: &mut Context<Self>) {
        self.mouse_down_pos = Some(pos);
        cx.notify();
    }

    fn mouse_up(&mut self, cx: &mut Context<Self>) {
        self.mouse_down_pos = None;
        cx.notify();
    }

    fn mouse_move(&mut self, pos: Point<Pixels>, cx: &mut Context<Self>) {
        if self.mouse_down_pos.is_none() {
            return;
        }

        let x = (pos.x - self.canvas_bounds.left()) / self.canvas_bounds.size.width;
        let y = (pos.y - self.canvas_bounds.top()) / self.canvas_bounds.size.height;

        self.value = [x.into(), y.into()];
        cx.notify();
    }
}

#[derive(IntoElement)]
pub struct PanTiltEditor {
    state: Entity<PanTiltEditorState>,

    paint_fn: Option<Box<dyn FnOnce(Bounds<Pixels>, Point<Pixels>, &mut Window, &mut App)>>,
}

impl PanTiltEditor {
    pub fn new(state: &Entity<PanTiltEditorState>) -> Self {
        Self {
            state: state.clone(),
            paint_fn: None,
        }
    }

    pub fn paint(
        mut self,
        paint_fn: impl FnOnce(Bounds<Pixels>, Point<Pixels>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.paint_fn = Some(Box::new(paint_fn));
        self
    }
}

impl RenderOnce for PanTiltEditor {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl gpui::IntoElement {
        div()
            .size_full()
            .child(
                canvas(
                    {
                        let state = self.state.clone();
                        move |bounds, _, cx| {
                            state.update(cx, |state, _| {
                                state.canvas_bounds = bounds;
                            });
                        }
                    },
                    {
                        let state = self.state.clone();
                        move |bounds, _, window, cx| {
                            let [x, y] = state.read(cx).value;

                            window.paint_quad(fill(bounds, black()));
                            window.paint_quad(outline(bounds, white(), gpui::BorderStyle::Solid));

                            let center_pos = bounds.map_clamped_point(&point(x, y));
                            let center_size = px(10.0);
                            let center_corner_radius = center_size / 2.0;

                            window.paint_quad(PaintQuad {
                                bounds: Bounds::centered_at(
                                    center_pos,
                                    size(center_size, center_size),
                                ),
                                corner_radii: Corners::all(center_corner_radius),
                                background: cx.theme().blue.into(),
                                border_widths: Edges::all(px(0.0)),
                                border_color: transparent_black(),
                                border_style: gpui::BorderStyle::Solid,
                            });

                            if let Some(paint_fn) = self.paint_fn {
                                paint_fn(bounds, center_pos, window, cx);
                            }
                        }
                    },
                )
                .size_64(),
            )
            .on_mouse_down(MouseButton::Left, {
                let state = self.state.clone();
                move |evt, _, cx| state.update(cx, |state, cx| state.mouse_down(evt.position, cx))
            })
            .on_mouse_up(MouseButton::Left, {
                let state = self.state.clone();
                move |_, _, cx| state.update(cx, |state, cx| state.mouse_up(cx))
            })
            .on_mouse_move({
                let state = self.state.clone();
                move |evt, _, cx| state.update(cx, |state, cx| state.mouse_move(evt.position, cx))
            })
    }
}
