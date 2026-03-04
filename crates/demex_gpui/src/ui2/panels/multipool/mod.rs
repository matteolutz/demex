use std::collections::HashMap;

use demex_core::{event::DemexEvent, pool::PoolType};
use gpui::{
    App, AppContext, BorderStyle, Bounds, Context, Entity, EventEmitter, FocusHandle, Focusable,
    Hsla, InteractiveElement, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, PaintQuad, ParentElement, Pixels, Point, Render, Styled, Subscription, Window,
    canvas, div, point, prelude::FluentBuilder, px, size,
};
use gpui_component::{
    ActiveTheme, Colorize, Disableable, IconName, StyledExt,
    button::Button,
    dock::{PanelEvent, PanelInfo, PanelState},
    v_flex, white,
};

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::{
        ext::GpuiContextExtension,
        panels::{
            DemexPanel,
            multipool::config::{MultiPoolConfig, MultiPoolEntry},
            pool::{
                pool_action::{apply_pool_type_to_button, handle_pool_item_click},
                pool_button::{PoolButton, PoolItemButtonIndicatorColor},
                pool_item::{PoolItemNameExt, PoolItemState},
                pool_quick_actions::PoolQuickActionsState,
                pool_type::PoolTypeExt,
            },
        },
        utils::bounds,
        window::add_pool_window::AddPoolWindow,
        wm::{WindowManager, edit_window::WindowManagerExtension},
    },
};

pub mod config;

const ELEMENT_PADDING: f32 = 5.0;
const ELEMENT_SIZE: f32 = 80.0;

mod actions {
    use gpui::{App, KeyBinding};

    gpui::actions!([MultiPoolZoomOut, MultiPoolZoomIn]);

    pub const KEY_CONTEXT: &str = "demex-mutlipool";

    pub fn init(cx: &mut App) {
        cx.bind_keys([
            KeyBinding::new("secondary-k", MultiPoolZoomOut, Some(KEY_CONTEXT)),
            KeyBinding::new("secondary-l", MultiPoolZoomIn, Some(KEY_CONTEXT)),
        ]);
    }
}

pub(super) fn init(cx: &mut App) {
    actions::init(cx);
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum MultiPoolDraggingState {
    Pool {
        pool_idx: usize,
    },
    New {
        start_cell: (u16, u16),
        current_cell: (u16, u16),
    },
    None,
}

impl MultiPoolDraggingState {
    pub fn is_dragging_pool(&self, pool_idx: usize) -> bool {
        match self {
            MultiPoolDraggingState::Pool { pool_idx: idx } => *idx == pool_idx,
            _ => false,
        }
    }

    pub fn is_new(&self) -> Option<((u16, u16), (u16, u16))> {
        match self {
            MultiPoolDraggingState::New {
                start_cell,
                current_cell,
            } => {
                let start_cell = (
                    start_cell.0.min(current_cell.0),
                    start_cell.1.min(current_cell.1),
                );
                let end_cell = (
                    start_cell.0.max(current_cell.0),
                    start_cell.1.max(current_cell.1),
                );

                Some((start_cell, end_cell))
            }
            _ => None,
        }
    }
}

pub struct MultiPoolPanel {
    focus_handle: FocusHandle,

    config: MultiPoolConfig,

    quick_actions_state: Entity<PoolQuickActionsState>,

    pool_item_states: Entity<HashMap<PoolType, HashMap<u32, PoolItemState>>>,

    element_size: Entity<f32>,

    bounds: Entity<Option<Bounds<Pixels>>>,

    current_dragging: Entity<MultiPoolDraggingState>,

    _subscriptions: Vec<Subscription>,
}

impl MultiPoolPanel {
    pub fn new(config: MultiPoolConfig, cx: &mut Context<Self>) -> Self {
        let bounds = cx.new(|_| None);
        let quick_actions_state = cx.new(|_| PoolQuickActionsState::default());

        let mut _subscriptions = vec![
            cx.observe_and_notify(&bounds),
            cx.observe_and_notify(&quick_actions_state),
        ];

        _subscriptions.extend(config.pool_types().iter().flat_map(|pool_type| {
            let pool_items = DemexUiState::pool(*pool_type, cx);

            [cx.observe_and_notify(&pool_items)]
                .into_iter()
                .chain(Self::get_pool_type_subscriptions(*pool_type, cx).into_iter())
        }));

        Self {
            focus_handle: cx.focus_handle(),

            config,
            element_size: cx.new(|_| ELEMENT_SIZE),

            quick_actions_state,

            pool_item_states: cx.new(|_| HashMap::new()),

            bounds,

            current_dragging: cx.new(|_| MultiPoolDraggingState::None),

            _subscriptions,
        }
    }

    pub fn add_pool(
        &mut self,
        pool_type: PoolType,
        start_cell: (u16, u16),
        end_cell: (u16, u16),
        start_id: u32,
        cx: &mut Context<Self>,
    ) {
        self.config.pools.push(MultiPoolEntry {
            pool_type,
            start_cell,
            size: (end_cell.0 - start_cell.0, end_cell.1 - start_cell.1),
            start_id,
        });
        cx.notify();
    }

    pub fn remove_pool(&mut self, pool_idx: usize, cx: &mut Context<Self>) {
        self.config.pools.remove(pool_idx);
        cx.notify();
    }

    fn clear_states(&self, pool_type: &PoolType, cx: &mut Context<Self>) {
        self.pool_item_states.update(cx, |states, _| {
            if let Some(states) = states.get_mut(pool_type) {
                states.clear();
            }
        });
        cx.notify();
    }

    fn clear_and_set_state(
        &self,
        pool_type: PoolType,
        id: u32,
        state: PoolItemState,
        cx: &mut Context<Self>,
    ) {
        self.pool_item_states.update(cx, |states, _| {
            let states = states.entry(pool_type).or_default();
            states.clear();
            states.insert(id, state);
        });
        cx.notify();
    }

    fn modify_state(
        &self,
        pool_type: PoolType,
        id: u32,
        f: impl FnOnce(Option<PoolItemState>) -> Option<PoolItemState>,
        cx: &mut Context<Self>,
    ) {
        self.pool_item_states.update(cx, |states, _| {
            let states = states.entry(pool_type).or_default();
            let state = states.remove(&id);
            if let Some(new_state) = f(state) {
                states.insert(id, new_state);
            };
        });
        cx.notify();
    }

    fn get_pool_type_subscriptions(
        pool_type: PoolType,
        cx: &mut Context<Self>,
    ) -> Vec<Subscription> {
        match pool_type {
            PoolType::Group => {
                vec![cx.observe(
                    &DemexUiState::fixture_selection(cx),
                    move |this, selection, cx| {
                        if let Some(group_id) =
                            selection.read(cx).as_ref().and_then(|sel| sel.group_id())
                        {
                            this.clear_and_set_state(
                                pool_type,
                                group_id,
                                PoolItemState {
                                    indicator_color: PoolItemButtonIndicatorColor::Green,
                                },
                                cx,
                            );
                        } else {
                            this.clear_states(&pool_type, cx);
                        }

                        cx.notify();
                    },
                )]
            }
            PoolType::Executor => {
                let sequence_pool = DemexUiState::pool(PoolType::Sequence, cx);

                vec![
                    cx.subscribe(
                        &DemexEngineHandler::event_handler(cx),
                        move |this, _, evt, cx| match evt {
                            DemexEvent::ExecutorGo(id) => {
                                this.modify_state(
                                    pool_type,
                                    *id,
                                    |_| {
                                        Some(PoolItemState {
                                            indicator_color: PoolItemButtonIndicatorColor::Red,
                                        })
                                    },
                                    cx,
                                );
                            }
                            DemexEvent::ExecutorStop(id) => {
                                this.modify_state(pool_type, *id, |_| None, cx);
                            }
                            _ => {}
                        },
                    ),
                    cx.observe_and_notify(&sequence_pool),
                ]
            }
            PoolType::Sequence => {
                vec![cx.observe(
                    &DemexUiState::selected_sequence(cx),
                    move |this, selected_sequence, cx| {
                        if let Some(selected_seq) = selected_sequence.read(cx) {
                            this.clear_and_set_state(
                                pool_type,
                                *selected_seq,
                                PoolItemState {
                                    indicator_color: PoolItemButtonIndicatorColor::Blue,
                                },
                                cx,
                            );
                        } else {
                            this.clear_states(&pool_type, cx);
                        }
                    },
                )]
            }
            _ => vec![],
        }
    }
}

impl EventEmitter<PanelEvent> for MultiPoolPanel {}
impl Focusable for MultiPoolPanel {
    fn focus_handle(&self, _: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl DemexPanel for MultiPoolPanel {
    fn panel_type() -> super::DockWindowPanelType {
        super::DockWindowPanelType::Multipool
    }

    fn dump(&self, state: &mut PanelState, _: &App) {
        state.info = PanelInfo::Panel(
            serde_json::to_value(self.config.clone()).unwrap_or(serde_json::Value::Null),
        );
    }

    fn deserialize(
        _dock_area: gpui::WeakEntity<gpui_component::dock::DockArea>,
        _panel_state: &PanelState,
        panel_info: &PanelInfo,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let pool_config = if let PanelInfo::Panel(panel) = panel_info {
            serde_json::from_value(panel.clone()).ok()
        } else {
            None
        };

        MultiPoolPanel::new(pool_config.unwrap_or_default(), cx)
    }
}

impl MultiPoolPanel {
    fn zoom_out<T>(&mut self, _: &T, _: &mut Window, cx: &mut Context<Self>) {
        self.element_size.update(cx, |size, _| *size -= 1.0);
        cx.notify();
    }

    fn zoom_in<T>(&mut self, _: &T, _: &mut Window, cx: &mut Context<Self>) {
        self.element_size.update(cx, |size, _| *size += 1.0);
        cx.notify();
    }

    fn render_dot_grid(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            cx.draw_canvas(|this, bounds, _, window, cx| {
                let element_size = this.element_size.read(cx);
                let (n_cols, n_rows) = this.config.size;

                for col in 0..n_cols {
                    for row in 0..n_rows {
                        let pos = bounds.origin
                            + point(px(element_size * col as f32), px(element_size * row as f32));
                        let size = size(px(2.5), px(2.5));

                        window.paint_quad(PaintQuad {
                            bounds: Bounds::centered_at(pos, size),
                            corner_radii: (size.width.as_f32() / 2.0).into(),
                            background: cx.theme().foreground.darken(0.5).into(),
                            border_widths: 0.0.into(),
                            border_color: Hsla::transparent_black(),
                            border_style: BorderStyle::Solid,
                        });
                    }
                }
            }),
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full()
    }

    fn get_mouse_grid_pos(
        &self,
        mouse_pos: &Point<Pixels>,
        cx: &Context<Self>,
    ) -> Option<(u16, u16)> {
        let Some(bounds) = self.bounds.read(cx) else {
            return None;
        };

        let Some(grid_pos) = bounds.localize(&mouse_pos) else {
            return None;
        };

        let element_size = self.element_size.read(cx);

        let mouse_grid_pos = (
            (grid_pos.x.as_f32() / element_size) as u16,
            (grid_pos.y.as_f32() / element_size) as u16,
        );

        Some(mouse_grid_pos)
    }

    fn show_new_pool_context_menu(
        &self,
        start_cell: (u16, u16),
        end_cell: (u16, u16),
        mouse_pos: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let window = window.window_handle();
        let this_pool = cx.entity();

        cx.defer(move |cx| {
            let _ = WindowManager::update_dock_window_handle(window, cx, |dw, _, cx| {
                dw.context_layer().update(cx, |context_layer, cx| {
                    context_layer.context_menu(
                        mouse_pos,
                        [("Add Pool", move |_: &mut Window, cx: &mut App| {
                            let this_pool = this_pool.clone();
                            WindowManager::open_edit_window::<AddPoolWindow>(
                                cx,
                                move |window, cx| {
                                    let this_pool = this_pool.clone();
                                    AddPoolWindow::new(this_pool, start_cell, end_cell, window, cx)
                                },
                            );
                        })],
                        cx,
                    );
                })
            });
        });
    }

    fn show_pool_context_menu(
        &self,
        pool_idx: usize,
        mouse_pos: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let window = window.window_handle();
        let this_pool = cx.entity();

        cx.defer(move |cx| {
            let _ = WindowManager::update_dock_window_handle(window, cx, |dw, _, cx| {
                dw.context_layer().update(cx, |context_layer, cx| {
                    context_layer.context_menu(
                        mouse_pos,
                        [("Delete", move |_: &mut Window, cx: &mut App| {
                            let this_pool = this_pool.clone();
                            this_pool.update(cx, |pool, cx| {
                                pool.remove_pool(pool_idx, cx);
                            });
                        })],
                        cx,
                    );
                })
            });
        });
    }

    fn render_grid(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let element_size = *self.element_size.read(cx);
        let (n_cols, n_rows) = self.config.size;

        div()
            .absolute()
            .top_0()
            .left_0()
            .w(px(n_cols as f32 * element_size))
            .h(px(n_rows as f32 * element_size))
            .grid()
            .grid_cols(n_cols)
            .grid_rows(n_rows)
            .gap(px(ELEMENT_PADDING))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, evt: &MouseUpEvent, window, cx| {
                    match this.current_dragging.read(cx) {
                        &MultiPoolDraggingState::New {
                            start_cell,
                            current_cell,
                        } if start_cell != current_cell => {
                            this.show_new_pool_context_menu(
                                start_cell,
                                current_cell,
                                evt.position,
                                window,
                                cx,
                            );
                        }
                        _ => {}
                    }

                    this.current_dragging.update(cx, |pool, _| {
                        *pool = MultiPoolDraggingState::None;
                    });
                    cx.notify();
                }),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, evt: &MouseDownEvent, _, cx| {
                    if let Some(mouse_grid_pos) = this.get_mouse_grid_pos(&evt.position, cx) {
                        this.current_dragging.update(cx, |curent_dragging, _| {
                            *curent_dragging = MultiPoolDraggingState::New {
                                start_cell: mouse_grid_pos,
                                current_cell: mouse_grid_pos,
                            };
                        });
                        cx.notify();
                    }
                }),
            )
            .on_mouse_move(cx.listener(|this, evt: &MouseMoveEvent, _, cx| {
                if let Some(mouse_grid_pos) = this.get_mouse_grid_pos(&evt.position, cx) {
                    match *this.current_dragging.read(cx) {
                        MultiPoolDraggingState::Pool { pool_idx } => {
                            this.config.pools[pool_idx].resize(mouse_grid_pos);

                            cx.notify();
                        }
                        MultiPoolDraggingState::New { start_cell, .. } => {
                            if let Some(mouse_cell) = this.get_mouse_grid_pos(&evt.position, cx) {
                                this.current_dragging.update(cx, |current_dragging, _| {
                                    *current_dragging = MultiPoolDraggingState::New {
                                        start_cell,
                                        current_cell: mouse_cell,
                                    };
                                });
                                cx.notify();
                            }
                        }
                        _ => {}
                    }
                }
            }))
            .children(self.config.pools.iter().enumerate().map(|(pool_idx, p)| {
                let pool = DemexUiState::pool(p.pool_type, cx).read(cx);

                let pool_type = p.pool_type;
                let pool_size = p.size;

                div()
                    .col_start(p.start_cell.0 as i16 + 1)
                    .col_end(p.start_cell.0 as i16 + 1 + pool_size.0 as i16)
                    .row_start(p.start_cell.1 as i16 + 1)
                    .row_end(p.start_cell.1 as i16 + 1 + pool_size.1 as i16)
                    .relative()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .when(
                        self.current_dragging.read(cx).is_dragging_pool(pool_idx),
                        |this| this.border_1().border_color(cx.theme().drag_border),
                    )
                    .child(
                        div()
                            .size_full()
                            .grid_cols(pool_size.0)
                            .grid_rows(pool_size.1)
                            .grid()
                            .bg(cx.theme().accent.blend(Hsla::black().alpha(0.2)))
                            .gap_1()
                            .rounded_lg()
                            .overflow_hidden()
                            .child(
                                div()
                                    .size_full()
                                    .flex()
                                    .justify_center()
                                    .items_center()
                                    .text_sm()
                                    .font_bold()
                                    .on_mouse_down(
                                        MouseButton::Right,
                                        cx.listener(
                                            move |this, evt: &MouseDownEvent, window, cx| {
                                                this.show_pool_context_menu(
                                                    pool_idx,
                                                    evt.position,
                                                    window,
                                                    cx,
                                                );
                                            },
                                        ),
                                    )
                                    .bg(p.pool_type.color(cx))
                                    .child(p.pool_type.to_short_string()),
                            )
                            .children(
                                (p.start_id..(p.start_id + p.num_items() as u32))
                                    .enumerate()
                                    .map(|(idx, id)| {
                                        let (col, row) = p.get_cell(idx);
                                        let pool_item = pool.iter().find(|p| p.id == id);

                                        PoolButton::new(format!("pool-{}-item{}", pool_idx, id))
                                            .col_start(col as i16 + 1)
                                            .col_end(col as i16 + 2)
                                            .row_start(row as i16 + 1)
                                            .row_end(row as i16 + 2)
                                            .quick_actions_state(&self.quick_actions_state)
                                            .when(true, |this| {
                                                apply_pool_type_to_button(
                                                    p.pool_type,
                                                    this,
                                                    pool_item,
                                                    id,
                                                )
                                            })
                                            .item_id(id)
                                            .when_some(pool_item, |this, pool_item| {
                                                this.item_name(pool_item.name.clone().to_name(cx))
                                                    .on_click({
                                                        move |_, _, cx| {
                                                            handle_pool_item_click(
                                                                pool_type, id, cx,
                                                            )
                                                        }
                                                    })
                                                    .when_some(
                                                        pool_item.colors.as_ref(),
                                                        |this, colors| {
                                                            this.colors_rgb(colors.iter().copied())
                                                        },
                                                    )
                                            })
                                            .when_some(
                                                self.pool_item_states
                                                    .read(cx)
                                                    .get(&p.pool_type)
                                                    .and_then(|states| states.get(&id)),
                                                |this, state| {
                                                    this.indicator_color(state.indicator_color)
                                                },
                                            )
                                            .when_none(&pool_item, |this| this.disabled(true))
                                            .into_any_element()
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .absolute()
                            .bottom_0()
                            .right_0()
                            .w_5()
                            .h_5()
                            .rounded_md()
                            .bg(cx.theme().secondary.lighten(0.5))
                            .cursor_nwse_resize()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    cx.stop_propagation();
                                    this.current_dragging.update(cx, |pool, _| {
                                        *pool = MultiPoolDraggingState::Pool { pool_idx };
                                    });
                                    cx.notify();
                                }),
                            )
                            .child(IconName::ResizeCorner),
                    )
            }))
            .when_some(
                self.current_dragging.read(cx).is_new(),
                |this, (start_cell, end_cell)| {
                    this.child(
                        div()
                            .col_start(start_cell.0 as i16 + 1)
                            .col_end(end_cell.0 as i16 + 1)
                            .row_start(start_cell.1 as i16 + 1)
                            .row_end(end_cell.1 as i16 + 1)
                            .bg(white().alpha(0.5))
                            .border_1()
                            .border_color(cx.theme().drag_border),
                    )
                },
            )
    }
}

impl Render for MultiPoolPanel {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> impl gpui::IntoElement {
        v_flex()
            .size_full()
            .p_2()
            .gap_1()
            .relative()
            .key_context(actions::KEY_CONTEXT)
            .on_action::<actions::MultiPoolZoomOut>(cx.listener(Self::zoom_out))
            .on_action::<actions::MultiPoolZoomIn>(cx.listener(Self::zoom_in))
            .child(bounds(&self.bounds).absolute().top_0().left_0().size_full())
            .child(
                div()
                    .w_full()
                    .h_12()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("zoom-out")
                            .icon(IconName::Minus)
                            .on_click(cx.listener(Self::zoom_out)),
                    )
                    .child(
                        Button::new("zoom-in")
                            .icon(IconName::Plus)
                            .on_click(cx.listener(Self::zoom_in)),
                    ),
            )
            .child(
                div()
                    .size_full()
                    .relative()
                    .child(self.render_dot_grid(window, cx))
                    .child(self.render_grid(window, cx)),
            )
        /*
        .when_some(self.bounds.read(cx).clone(), |this, bounds| {
            this.child(PoolQuickActions::new(&self.quick_actions_state, bounds))
        })*/
    }
}
