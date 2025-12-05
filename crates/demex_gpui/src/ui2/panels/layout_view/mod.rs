use std::cmp::Ordering;

use demex_core::command::parser::nodes::action::Action;
use gpui::{
    App, AppContext, BorderStyle, Bounds, Context, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    PaintQuad, ParentElement, Pixels, Point, Render, ScrollWheelEvent, Styled, Subscription,
    Window, black, canvas, div, fill, px, white,
};
use gpui_component::{
    PixelsExt,
    button::Button,
    dock::{Panel, PanelEvent, register_panel},
    v_flex,
};
use itertools::Itertools;

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::{
        ext::GpuiContextExtension,
        panels::layout_view::{
            layout_entry::{FixtureLayoutEntryDrawArgs, FixtureLayoutEntryExt},
            layout_projection::{LayoutProjection, PosExt},
        },
    },
};

pub mod layout_entry;
pub mod layout_projection;

const LAYOUT_VIEW_PANEL_NAME: &str = "layout-view";

pub(super) fn register(cx: &mut App) {
    register_panel(cx, LAYOUT_VIEW_PANEL_NAME, |_, _, _, window, cx| {
        Box::new(cx.new(|cx| LayoutViewPanel::new(window, cx)))
    });
}

pub struct LayoutViewPanel {
    focus_handle: FocusHandle,

    screen_bounds: Entity<Bounds<Pixels>>,

    projection: Entity<LayoutProjection>,

    last_middle_button_mouse_pos: Entity<Option<Point<Pixels>>>,
    selection_start_mouse_pos: Entity<Option<Point<Pixels>>>,

    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<PanelEvent> for LayoutViewPanel {}
impl Focusable for LayoutViewPanel {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for LayoutViewPanel {
    fn panel_name(&self) -> &'static str {
        LAYOUT_VIEW_PANEL_NAME
    }

    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        "Layout View"
    }
}

impl LayoutViewPanel {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let subs = vec![
            cx.observe_and_notify(&DemexUiState::patch(cx)),
            cx.observe_and_notify(&DemexUiState::fixture_selection(cx)),
        ];

        let screen_bounds = cx.new(|_| Bounds::default());
        let projection = cx.new(|_| LayoutProjection::new(screen_bounds.clone()));

        Self {
            focus_handle: cx.focus_handle(),
            screen_bounds,
            projection,
            last_middle_button_mouse_pos: cx.new(|_| None),
            selection_start_mouse_pos: cx.new(|_| None),
            _subscriptions: subs,
        }
    }
}

impl LayoutViewPanel {
    fn handle_middle_mouse_button_down(
        &mut self,
        evt: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.last_middle_button_mouse_pos
            .update(cx, |pos, _| *pos = Some(evt.position));
        cx.notify();
    }

    fn handle_middle_mouse_button_up(
        &mut self,
        _evt: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.last_middle_button_mouse_pos
            .update(cx, |pos, _| *pos = None);
        cx.notify();
    }

    fn handle_left_mouse_button_down(
        &mut self,
        evt: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.selection_start_mouse_pos
            .update(cx, |pos, _| *pos = Some(evt.position));
        cx.notify();
    }

    fn handle_left_mouse_button_up(
        &mut self,
        evt: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(start_pos) = *self.selection_start_mouse_pos.read(cx) else {
            return;
        };

        let world_selection_origin = self.projection.read(cx).unproject(start_pos, cx);

        let selection_bounds =
            Bounds::from_corners(start_pos.min(&evt.position), start_pos.max(&evt.position));

        let unprojected_selection_bounds = self
            .projection
            .read(cx)
            .unproject_bounds(selection_bounds, cx);

        let selected_fixtures = DemexUiState::patch(cx)
            .read(cx)
            .layout()
            .fixtures()
            .iter()
            .filter(|fixture| {
                unprojected_selection_bounds.contains(&fixture.position().to_gpui_point())
            })
            .sorted_by(|a, b| {
                a.position()
                    .gpui_distance_to(&world_selection_origin)
                    .partial_cmp(&b.position().gpui_distance_to(&world_selection_origin))
                    .unwrap_or(Ordering::Equal)
            })
            .map(|fixture| fixture.fixture_id())
            .collect::<Vec<_>>();

        DemexEngineHandler::engine(cx).exec_ui(Action::AddFixturesToSelection(selected_fixtures));

        self.selection_start_mouse_pos
            .update(cx, |pos, _| *pos = None);
        cx.notify();
    }

    fn handle_mouse_move(
        &mut self,
        evt: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(last_pos) = *self.last_middle_button_mouse_pos.read(cx) {
            let projection = self.projection.read(cx);

            let from = projection.unproject(last_pos, cx);
            let to = projection.unproject(evt.position, cx);
            let delta = to - from;

            self.projection.update(cx, |proj, _| {
                *proj.center_mut() += delta;
            });
            self.last_middle_button_mouse_pos
                .update(cx, |pos, _| *pos = Some(evt.position));

            cx.notify();
        };

        if self.selection_start_mouse_pos.read(cx).is_some() {
            cx.notify();
        }
    }

    fn handle_scroll_wheel(
        &mut self,
        evt: &ScrollWheelEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.projection.update(cx, |proj, _| {
            let delta = evt.delta.pixel_delta(px(1.0));
            *proj.zoom_mut() += delta.y.as_f32() * 0.1;
        });
        cx.notify();
    }
}

impl LayoutViewPanel {
    fn draw_canvas(
        &mut self,
        bounds: Bounds<Pixels>,
        _: (),
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let layout = DemexUiState::patch(cx).read(cx).layout();
        let fixture_selection = DemexUiState::fixture_selection(cx).read(cx);

        window.paint_quad(fill(bounds, black()));

        for fixture in layout.fixtures() {
            let args = FixtureLayoutEntryDrawArgs {
                is_selected: fixture_selection
                    .as_ref()
                    .is_some_and(|fs| fs.has_fixture(fixture.fixture_id())),
            };
            fixture.draw(args, self.projection.read(cx), window, cx);
        }

        if let Some(selection_start_pos) = *self.selection_start_mouse_pos.read(cx) {
            let mouse_pos = window.mouse_position();

            let bounds = Bounds::from_corners(
                selection_start_pos.min(&mouse_pos),
                selection_start_pos.max(&mouse_pos),
            );

            window.paint_quad(PaintQuad {
                bounds: bounds.into(),
                corner_radii: (0.).into(),
                background: white().alpha(0.2).into(),
                border_widths: (1.).into(),
                border_color: white(),
                border_style: BorderStyle::default(),
            });
        }
    }
}

impl Render for LayoutViewPanel {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        let screen_bounds = self.screen_bounds.clone();

        v_flex()
            .size_full()
            .child(
                v_flex()
                    .p_4()
                    .gap_1()
                    .child("Layout View")
                    .child(format!(
                        "Zoom: {}, Center: {:?}",
                        self.projection.read(cx).zoom(),
                        self.projection.read(cx).center()
                    ))
                    .child(Button::new("reset").label("Reset").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.projection.update(cx, |proj, _| proj.reset());
                            cx.notify();
                        },
                    ))),
            )
            .child(
                div()
                    .size_full()
                    .cursor_crosshair()
                    .child(
                        canvas(
                            move |bounds, _, cx| {
                                screen_bounds.update(cx, |sb, _| {
                                    *sb = bounds;
                                });
                            },
                            cx.draw_canvas(Self::draw_canvas),
                        )
                        .size_full()
                        .overflow_hidden(),
                    )
                    .overflow_hidden()
                    .on_mouse_down(
                        MouseButton::Middle,
                        cx.listener(Self::handle_middle_mouse_button_down),
                    )
                    .on_mouse_up(
                        MouseButton::Middle,
                        cx.listener(Self::handle_middle_mouse_button_up),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(Self::handle_left_mouse_button_down),
                    )
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(Self::handle_left_mouse_button_up),
                    )
                    .on_mouse_move(cx.listener(Self::handle_mouse_move))
                    .on_scroll_wheel(cx.listener(Self::handle_scroll_wheel)),
            )
    }
}
