use gpui::{
    App, AppContext, Bounds, Context, Entity, EventEmitter, FocusHandle, Focusable, Hsla,
    InteractiveElement, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    ParentElement, Pixels, Point, Render, ScrollWheelEvent, Styled, Subscription, Window, canvas,
    div, fill, px, solid_background,
};
use gpui_component::{
    PixelsExt,
    button::Button,
    dock::{Panel, PanelEvent, register_panel},
    v_flex,
};

use crate::{
    engine::state::DemexUiState,
    ui2::{
        ext::GpuiContextExtension,
        panels::layout_view::{
            layout_entry::{FixtureLayoutEntryDrawArgs, FixtureLayoutEntryExt},
            layout_projection::LayoutProjection,
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
    last_mouse_pos: Entity<Option<Point<Pixels>>>,

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

    fn title(&self, _window: &Window, _cx: &App) -> gpui::AnyElement {
        "Layout View".into_any_element()
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
            last_mouse_pos: cx.new(|_| None),
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
        self.last_mouse_pos
            .update(cx, |pos, _| *pos = Some(evt.position));
        cx.notify();
    }

    fn handle_middle_mouse_button_up(
        &mut self,
        _evt: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.last_mouse_pos.update(cx, |pos, _| *pos = None);
        cx.notify();
    }

    fn handle_mouse_move(
        &mut self,
        evt: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(last_pos) = *self.last_mouse_pos.read(cx) else {
            return;
        };

        let projection = self.projection.read(cx);

        let from = projection.unproject(last_pos, cx);
        let to = projection.unproject(evt.position, cx);
        let delta = to - from;

        self.projection.update(cx, |proj, _| {
            *proj.center_mut() += delta;
        });
        self.last_mouse_pos
            .update(cx, |pos, _| *pos = Some(evt.position));

        cx.notify();
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

impl Render for LayoutViewPanel {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        let projection = self.projection.clone();

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
                            move |bounds, _, window, cx| {
                                let layout = DemexUiState::patch(cx).read(cx).layout();
                                let fixture_selection =
                                    DemexUiState::fixture_selection(cx).read(cx);

                                window.paint_quad(fill(bounds, solid_background(Hsla::black())));

                                for fixture in layout.fixtures() {
                                    let args = FixtureLayoutEntryDrawArgs {
                                        is_selected: fixture_selection
                                            .as_ref()
                                            .is_some_and(|fs| fs.has_fixture(fixture.fixture_id())),
                                    };
                                    fixture.draw(args, projection.read(cx), window, cx);
                                }
                            },
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
                    .on_mouse_move(cx.listener(Self::handle_mouse_move))
                    .on_scroll_wheel(cx.listener(Self::handle_scroll_wheel)),
            )
    }
}
