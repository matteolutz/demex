use std::cmp::Ordering;

use demex_core::command::parser::nodes::action::Action;
use gpui::{
    App, AppContext, BorderStyle, Bounds, Context, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    PaintQuad, ParentElement, Pixels, Point, Render, ScrollWheelEvent, Styled, Subscription,
    Window, black, canvas, div, fill, prelude::FluentBuilder, px, white,
};
use gpui_component::{
    PixelsExt,
    button::Button,
    dock::{Panel, PanelEvent, register_panel},
    h_flex,
    slider::{Slider, SliderEvent, SliderState},
    tab::TabBar,
    v_flex,
};
use itertools::Itertools;

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::{
        ext::GpuiContextExtension,
        panels::{
            layout_view::{
                layout_entry::{FixtureLayoutEntryDrawArgs, FixtureLayoutEntryExt},
                layout_projection::LayoutProjection,
            },
            toolbar_buttons,
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

    selected_layout: Entity<usize>,

    zoom_slider_state: Entity<SliderState>,

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

    fn inner_padding(&self, _cx: &App) -> bool {
        false
    }

    fn toolbar_buttons(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Vec<Button>> {
        Some(toolbar_buttons(self, window, cx))
    }
}

impl LayoutViewPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let initial_zoom = 1.0;

        let screen_bounds = cx.new(|_| Bounds::default());
        let projection =
            cx.new(|_| LayoutProjection::new(screen_bounds.clone()).with_zoom(initial_zoom));

        let zoom_slider_state = cx.new(|_| {
            SliderState::new()
                .default_value(initial_zoom)
                .min(0.1)
                .max(10.0)
                .step(0.01)
        });

        let selected_layout = cx.new(|_| 0);

        let subs = vec![
            cx.observe_and_notify(&DemexUiState::patch(cx)),
            cx.observe_and_notify(&DemexUiState::fixture_selection(cx)),
            cx.observe_and_notify(&DemexUiState::fixture_values(cx)),
            cx.observe_in(&projection, window, |this, projection, window, cx| {
                this.zoom_slider_state.update(cx, |state, cx| {
                    state.set_value(projection.read(cx).zoom(), window, cx);
                    cx.notify();
                });
            }),
            cx.observe(&selected_layout, move |this, _, cx| {
                this.projection.update(cx, |projection, _| {
                    projection.reset_with_zoom(initial_zoom);
                });
                cx.notify();
            }),
            cx.subscribe_in(
                &zoom_slider_state,
                window,
                |this, _, evt, _, cx| match evt {
                    SliderEvent::Change(value) => {
                        this.projection.update(cx, |projection, _| {
                            *projection.zoom_mut() = value.start();
                            // don't notify the projection
                        });
                        cx.notify();
                    }
                },
            ),
        ];

        Self {
            focus_handle: cx.focus_handle(),
            screen_bounds,
            projection,
            last_middle_button_mouse_pos: cx.new(|_| None),
            selection_start_mouse_pos: cx.new(|_| None),
            zoom_slider_state,
            selected_layout,
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
        // double click
        if evt.click_count == 2 {
            self.selection_start_mouse_pos
                .update(cx, |pos, _| *pos = None);
            self.last_middle_button_mouse_pos
                .update(cx, |pos, _| *pos = Some(evt.position));
            return;
        }

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
        if self.last_middle_button_mouse_pos.read(cx).is_some() {
            self.last_middle_button_mouse_pos
                .update(cx, |pos, _| *pos = None);
            cx.notify();
        }

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

        let selected_layout = *self.selected_layout.read(cx);
        let selected_fixtures = DemexUiState::patch(cx).read(cx).layout_pool()[selected_layout]
            .fixtures()
            .iter()
            .flat_map(|fixture| fixture.get_draw_entries())
            .filter(|fixture| unprojected_selection_bounds.contains(&fixture.pos))
            .sorted_by(|a, b| {
                a.pos
                    .relative_to(&world_selection_origin)
                    .magnitude()
                    .partial_cmp(&b.pos.relative_to(&world_selection_origin).magnitude())
                    .unwrap_or(Ordering::Equal)
            })
            .map(|fixture| fixture.fixture_path)
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

            self.projection.update(cx, |proj, cx| {
                *proj.center_mut() += delta;
                cx.notify();
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
        self.projection.update(cx, |proj, cx| {
            let delta = evt.delta.pixel_delta(px(1.0));
            *proj.zoom_mut() += delta.y.as_f32() * 0.01;
            cx.notify();
        });
        cx.notify();
    }

    fn handle_tab_clicked(&mut self, tab: &usize, _: &mut Window, cx: &mut Context<Self>) {
        self.selected_layout.update(cx, |selected_layout, cx| {
            *selected_layout = *tab;
            cx.notify();
        });
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
        let selected_layout = *self.selected_layout.read(cx);

        // TODO: fix this
        let layout = &DemexUiState::patch(cx).read(cx).layout_pool()[selected_layout].clone();

        let fixture_selection = DemexUiState::fixture_selection(cx)
            .read(cx)
            .as_ref()
            .map(|fs| fs.selection().clone());

        window.paint_quad(fill(bounds, black()));

        let args = FixtureLayoutEntryDrawArgs {
            selection: fixture_selection.as_ref(),
        };

        for fixture in layout.fixtures() {
            fixture.draw(args.clone(), &self.projection, window, cx);
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

        let layouts = DemexUiState::patch(cx).read(cx).layout_pool();
        let selected_layout = *self.selected_layout.read(cx);

        v_flex()
            .size_full()
            .child(
                TabBar::new("layout-selector")
                    .children(layouts.iter().map(|l| l.name().to_string()))
                    .selected_index(selected_layout)
                    .on_click(cx.listener(Self::handle_tab_clicked)),
            )
            .child(
                h_flex()
                    .w_full()
                    .items_center()
                    .gap_2()
                    .px_4()
                    .py_2()
                    .child(Slider::new(&self.zoom_slider_state).horizontal())
                    .child(Button::new("reset").label("Reset").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.projection.update(cx, |proj, cx| {
                                proj.reset();
                                cx.notify();
                            });
                            cx.notify();
                        },
                    ))),
            )
            .child(
                div()
                    .size_full()
                    .cursor_crosshair()
                    .when(
                        self.last_middle_button_mouse_pos.read(cx).is_some(),
                        |this| this.cursor_grabbing(),
                    )
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
