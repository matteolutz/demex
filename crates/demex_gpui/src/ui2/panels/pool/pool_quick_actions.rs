use std::{collections::HashMap, rc::Rc, time::Duration};

use gpui::{
    AnyWindowHandle, App, Bounds, BoxShadow, ElementId, Entity, InteractiveElement, IntoElement,
    ParentElement, Pixels, Point, RenderOnce, SharedString, Styled, Task, Window, div, point,
    prelude::FluentBuilder, px,
};
use gpui_component::{ActiveTheme, StyledExt, v_flex};

use crate::ui2::{
    components::context::{DemexContextLayerContent, DemexContextLayerContentMode},
    wm::WindowManager,
};

const QUICK_ACTIONS_TIMEOUT: f32 = 0.5;
const QUICK_ACTIONS_MOUSE_MOVE_THRESHOLD: f64 = 5.0;

#[derive(Clone)]
pub struct PoolQuickAction {
    pub(super) name: SharedString,
    pub(super) action: Rc<dyn Fn(&mut Window, &mut App)>,
}

struct CurrentPoolButton {
    id: ElementId,

    // should_display: bool,
    timer: Option<Task<()>>,

    mouse_down_pos: Point<Pixels>,
    actions: Vec<Option<PoolQuickAction>>,
}

impl CurrentPoolButton {}

#[derive(Default)]
pub struct PoolQuickActionsState {
    current_pool_button: Option<CurrentPoolButton>,

    pool_button_bounds: HashMap<ElementId, Bounds<Pixels>>,
}

pub(super) trait PoolQuickActionsStateEntityExtension {
    fn submit_context(&self, window_handle: AnyWindowHandle, cx: &mut App);
    fn clear_context(&self, window_handle: AnyWindowHandle, cx: &mut App);

    fn mouse_down(
        &self,
        pos: Point<Pixels>,
        actions: Vec<Option<PoolQuickAction>>,
        id: ElementId,
        window: &mut Window,
        cx: &mut App,
    );
    fn mouse_move(&self, pos: Point<Pixels>, id: ElementId, window: &mut Window, cx: &mut App);
    fn mouse_up(&self, _position: Point<Pixels>, id: ElementId, window: &mut Window, cx: &mut App);
}

impl PoolQuickActionsStateEntityExtension for Entity<PoolQuickActionsState> {
    fn submit_context(&self, window_handle: AnyWindowHandle, cx: &mut App) {
        let state = self.clone();

        cx.defer(move |cx| {
            let _ = WindowManager::update_dock_window_handle(window_handle, cx, |dw, _, cx| {
                dw.context_layer().update(cx, |layer, cx| {
                    layer.set_content(
                        DemexContextLayerContent {
                            content_mode: DemexContextLayerContentMode::QuickActions { state },
                            pos: Point::default(),
                        },
                        cx,
                    );
                })
            });
        });
    }

    fn clear_context(&self, window_handle: AnyWindowHandle, cx: &mut App) {
        cx.defer(move |cx| {
            let _ = WindowManager::update_dock_window_handle(window_handle, cx, |dw, _, cx| {
                dw.context_layer().update(cx, |layer, cx| {
                    layer.clear(cx);
                })
            });
        });
    }

    fn mouse_down(
        &self,
        pos: Point<Pixels>,
        actions: Vec<Option<PoolQuickAction>>,
        id: ElementId,
        window: &mut Window,
        cx: &mut App,
    ) {
        let entity = self.clone();

        self.update(cx, move |this, cx| {
            if actions.is_empty() {
                return;
            }

            let timer = cx.spawn_in(window, {
                let id = id.clone();
                async move |this, cx| {
                    let window_handle = cx.window_handle();

                    cx.background_executor()
                        .timer(Duration::from_secs_f32(QUICK_ACTIONS_TIMEOUT))
                        .await;

                    let _ = this.update(cx, |state, cx| {
                        let Some(current_pool_button) = state.current_pool_button.as_mut() else {
                            return;
                        };

                        if current_pool_button.id != id {
                            return;
                        }

                        // current_pool_button.should_display = true;
                        entity.submit_context(window_handle, cx);
                        cx.notify();
                    });
                }
            });

            this.current_pool_button = Some(CurrentPoolButton {
                id,
                mouse_down_pos: pos,
                actions,
                timer: Some(timer),
            });
        });
    }

    fn mouse_move(&self, pos: Point<Pixels>, id: ElementId, window: &mut Window, cx: &mut App) {
        self.update(cx, |this, cx| {
            let Some(current_button) = this.current_pool_button.as_mut() else {
                return;
            };

            if current_button.id != id {
                return;
            }

            if current_button.mouse_down_pos.relative_to(&pos).magnitude()
                < QUICK_ACTIONS_MOUSE_MOVE_THRESHOLD
            {
                return;
            }

            // when the mouse is moved while already being held down,
            // immediately display the quick actions and stop the timer
            let _ = current_button.timer.take();
            self.submit_context(window.window_handle(), cx);

            cx.notify();
        });
    }

    fn mouse_up(&self, _position: Point<Pixels>, id: ElementId, window: &mut Window, cx: &mut App) {
        self.update(cx, |this, cx| {
            let Some(current_button) = this.current_pool_button.as_ref() else {
                return;
            };

            if current_button.id != id {
                return;
            }

            this.current_pool_button = None;
            self.clear_context(window.window_handle(), cx);

            cx.notify();
        });
    }
}

impl PoolQuickActionsState {
    fn current_pool_button_bounds(&self) -> Option<Bounds<Pixels>> {
        self.current_pool_button
            .as_ref()
            .and_then(|button| self.pool_button_bounds.get(&button.id).cloned())
    }

    pub(super) fn update_bounds(&mut self, id: ElementId, bounds: Bounds<Pixels>) {
        self.pool_button_bounds.insert(id, bounds);
    }
}

#[derive(IntoElement)]
pub struct PoolQuickActions {
    pub(crate) state: Entity<PoolQuickActionsState>,
    // container_bounds: Bounds<Pixels>,
}

impl PoolQuickActions {
    fn render_action_button(
        idx: usize,
        action: Option<&PoolQuickAction>,
        cx: &App,
    ) -> impl IntoElement {
        let on_action = action.map(|action| action.action.clone());

        v_flex()
            .id(idx)
            .bg(cx.theme().secondary)
            .border_1()
            .border_color(cx.theme().border)
            .when(idx == 0, |this| this.rounded_tl_sm()) // Top left
            .when(idx == 2, |this| this.rounded_tr_sm()) // Top right
            .when(idx == 5, |this| this.rounded_bl_sm()) // Bottom left
            .when(idx == 7, |this| this.rounded_br_sm()) // Bottom right
            //.rounded_sm()
            .when(action.is_some(), |this| {
                this.hover(|this| {
                    this.bg(cx.theme().secondary_active)
                        .border_color(cx.theme().primary)
                })
            })
            .size_full()
            .justify_center()
            .items_center()
            .when_some(on_action, |this, on_action| {
                this.on_mouse_up(gpui::MouseButton::Left, move |_, window, app| {
                    (on_action)(window, app)
                })
            })
            .p_1()
            .text_sm()
            .font_semibold()
            .text_center()
            .when_some(action, |this, action| this.child(action.name.clone()))
    }
}

impl RenderOnce for PoolQuickActions {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl gpui::IntoElement {
        let Some(button_bounds) = self.state.read(cx).current_pool_button_bounds() else {
            return div();
        };

        let current_button = self.state.read(cx).current_pool_button.as_ref().unwrap();

        let button_center = button_bounds.center();
        let container_size = button_bounds.size.width + px(80.0);

        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .block_mouse_except_scroll()
            .child(
                div()
                    .size(container_size)
                    .absolute()
                    .left(button_center.x - container_size / 2.0)
                    .top(button_center.y - container_size / 2.0)
                    .grid()
                    .gap_1()
                    .grid_rows(3)
                    .grid_cols(3)
                    .shadow(vec![BoxShadow {
                        color: cx.theme().background.alpha(0.75),
                        offset: point(px(0.0), px(0.0)),
                        blur_radius: px(20.0),
                        spread_radius: px(20.0),
                    }])
                    .children((0..4).map(|idx| {
                        let action = current_button.actions.get(idx).and_then(|idx| idx.as_ref());
                        Self::render_action_button(idx, action, cx)
                    }))
                    .child(div())
                    .children((4..8).map(|idx| {
                        let action = current_button.actions.get(idx).and_then(|idx| idx.as_ref());
                        Self::render_action_button(idx, action, cx)
                    })),
            )
    }
}
