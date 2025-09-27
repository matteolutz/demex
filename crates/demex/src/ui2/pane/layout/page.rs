use gpui::{
    App, Bounds, Canvas, Entity, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point,
    canvas, div, fill, outline, prelude::*, rgb,
};

use demex_ui::{
    AppExt,
    container::container,
    theme::ActiveTheme,
    utils::{SnapModeFunction, snap_point_2},
};
use gpui::{Render, point, px};

use crate::ui2::{
    pane::layout::element::LayoutViewElement,
    window::add_layout_item::{AddLayoutItemWindow, AddLayoutItemWindowInitData},
};

const GRID_N_COLS: u16 = 15;
const GRID_N_ROWS: u16 = 15;

pub struct LayoutViewPage {
    name: String,
    elements: Vec<LayoutViewElement>,
    selection: Option<(Point<Pixels>, Point<Pixels>)>,

    canvas_bounds: Entity<Option<Bounds<Pixels>>>,
}

impl LayoutViewPage {
    pub fn new(cx: &mut App, name: impl Into<String>, elements: Vec<LayoutViewElement>) -> Self {
        Self {
            name: name.into(),
            elements,
            selection: None,
            canvas_bounds: cx.new(|_| None),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl LayoutViewPage {
    fn cell_width(&self, cx: &mut gpui::Context<Self>) -> Pixels {
        let canvas_bounds = self.canvas_bounds.read(cx);
        canvas_bounds
            .map(|bounds| bounds.size.width / GRID_N_COLS as f32)
            .unwrap_or(px(0.0))
    }

    fn cell_height(&self, cx: &mut gpui::Context<Self>) -> Pixels {
        let canvas_bounds = self.canvas_bounds.read(cx);
        canvas_bounds
            .map(|bounds| bounds.size.height / GRID_N_ROWS as f32)
            .unwrap_or(px(0.0))
    }

    fn selection_on_grid(&self, cx: &mut gpui::Context<Self>) -> Option<(Point<u16>, Point<u16>)> {
        self.selection.and_then(|(from_point, to_point)| {
            let Some(bounds) = self.canvas_bounds.read(cx).clone() else {
                return None;
            };

            let cell_width = self.cell_width(cx);
            let cell_height = self.cell_height(cx);

            let threshold_point = point(cell_width, cell_height);

            let from_point_snapped = snap_point_2(
                from_point - bounds.origin,
                threshold_point,
                SnapModeFunction::Round,
            );
            let to_point_snapped = snap_point_2(
                to_point - bounds.origin,
                threshold_point,
                SnapModeFunction::Round,
            );

            Some((
                point(
                    (from_point_snapped.x.0 / cell_width.0) as u16,
                    (from_point_snapped.y.0 / cell_height.0) as u16,
                ),
                point(
                    (to_point_snapped.x.0 / cell_width.0) as u16,
                    (to_point_snapped.y.0 / cell_height.0) as u16,
                ),
            ))
        })
    }

    fn render_grid(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> Canvas<()> {
        let dot_color = cx.theme().accent;
        let selection_color = cx.theme().accent.opacity(0.4);
        let selection = self.selection;
        let bounds = self.canvas_bounds.clone();

        canvas(
            move |canvas_bounds, _, cx| {
                bounds.update(cx, |bounds, _| *bounds = Some(canvas_bounds));
            },
            {
                move |canvas_bounds, _, window, cx| {
                    let cell_width = canvas_bounds.size.width / GRID_N_COLS as f32;
                    let cell_height = canvas_bounds.size.height / GRID_N_ROWS as f32;

                    for x in 0..=(GRID_N_COLS) {
                        for y in 0..=(GRID_N_ROWS) {
                            window.paint_quad(fill(
                                Bounds::from_corner_and_size(
                                    gpui::Corner::TopLeft,
                                    point(x as f32 * cell_width, y as f32 * cell_height)
                                        + canvas_bounds.origin,
                                    gpui::size(px(2.0), px(2.0)),
                                ),
                                dot_color,
                            ));
                        }
                    }

                    if let Some((from_point, to_point)) = selection {
                        let threshold_point = point(cell_width, cell_height);

                        let from_point_snapped = snap_point_2(
                            from_point - canvas_bounds.origin,
                            threshold_point,
                            SnapModeFunction::Round,
                        );
                        let to_point_snapped = snap_point_2(
                            to_point - canvas_bounds.origin,
                            threshold_point,
                            SnapModeFunction::Round,
                        );

                        let mut selection_bounds =
                            Bounds::from_corners(from_point_snapped, to_point_snapped);
                        selection_bounds.origin += canvas_bounds.origin;

                        window.paint_quad(
                            fill(selection_bounds, selection_color).corner_radii(cx.theme().radius),
                        );

                        window.paint_quad(
                            outline(
                                Bounds::from_corners(from_point, to_point),
                                rgb(0xffffff),
                                gpui::BorderStyle::Dashed,
                            )
                            .corner_radii(cx.theme().radius),
                        );
                    }
                }
            },
        )
    }
}

impl LayoutViewPage {
    fn handle_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        self.selection = Some((event.position, event.position));
        cx.notify();
    }

    fn handle_mouse_up(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        if let Some(selection) = self.selection_on_grid(cx)
            && selection.0 != selection.1
        {
            cx.update_wm(|wm, cx| {
                wm.open_singleton_window::<AddLayoutItemWindow>(
                    cx,
                    AddLayoutItemWindowInitData { selection },
                );
            });
        }

        self.selection = None;
        cx.notify();
    }

    fn handle_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        let Some((_, ref mut selection_end)) = self.selection else {
            return;
        };

        *selection_end = event.position;
        cx.notify();
    }
}

impl Render for LayoutViewPage {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div()
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(Self::handle_mouse_down),
            )
            .on_mouse_up(gpui::MouseButton::Left, cx.listener(Self::handle_mouse_up))
            .on_mouse_move(cx.listener(Self::handle_mouse_move))
            .cursor_crosshair()
            .relative()
            .child(self.render_grid(window, cx).absolute().size_full())
            .child(
                div()
                    .absolute()
                    .size_full()
                    .grid()
                    .grid_cols(GRID_N_COLS as u16)
                    .grid_rows(GRID_N_ROWS as u16)
                    .children(self.elements.iter().enumerate().map(|(idx, el)| {
                        container(window, cx)
                            .cursor_default()
                            .occlude()
                            .col_start(el.from.x as i16 + 1)
                            .row_start(el.from.y as i16 + 1)
                            .col_end(el.to.x as i16 + 1)
                            .row_end(el.to.y as i16 + 1)
                            .child(el.element_type.render())
                    })),
            )
            .size_full()
    }
}
