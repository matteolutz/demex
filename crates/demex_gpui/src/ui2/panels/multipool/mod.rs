use std::collections::HashMap;

use demex_core::{event::DemexEvent, pool::PoolType};
use gpui::{
    App, AppContext, BorderStyle, Bounds, Context, Entity, EventEmitter, FocusHandle, Focusable,
    Hsla, InteractiveElement, IntoElement, MouseButton, MouseMoveEvent, PaintQuad, ParentElement,
    Pixels, Point, Render, Styled, Subscription, Window, canvas, div, point,
    prelude::FluentBuilder, px, size,
};
use gpui_component::{
    ActiveTheme, Colorize, Disableable, IconName, PixelsExt, StyledExt,
    button::Button,
    dock::{Panel, PanelEvent, PanelInfo, PanelState, register_panel},
    v_flex,
};

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::{
        ext::GpuiContextExtension,
        panels::{
            multipool::config::MultiPoolConfig,
            pool::{
                pool_action::{apply_pool_type_to_button, handle_pool_item_click},
                pool_button::{PoolButton, PoolItemButtonIndicatorColor},
                pool_item::{PoolItemNameExt, PoolItemState},
                pool_quick_actions::PoolQuickActionsState,
                pool_type::PoolTypeExt,
            },
            toolbar_buttons,
        },
        utils::bounds,
    },
};

pub mod config;

const MULTIPOOL_PANEL_NAME: &str = "demex-multipool";

const ELEMENT_PADDING: f32 = 5.0;
const ELEMENT_SIZE: f32 = 80.0;

pub(super) fn register(cx: &mut App) {
    register_panel(cx, MULTIPOOL_PANEL_NAME, |_, _, panel_info, _, cx| {
        let pool_config = if let PanelInfo::Panel(panel) = panel_info {
            serde_json::from_value(panel.clone()).ok()
        } else {
            None
        };

        Box::new(cx.new(|cx| MultiPoolPanel::new(pool_config.unwrap_or_default(), cx)))
    });
}

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

pub struct MultiPoolPanel {
    focus_handle: FocusHandle,

    config: MultiPoolConfig,

    quick_actions_state: Entity<PoolQuickActionsState>,

    pool_item_states: Entity<HashMap<PoolType, HashMap<u32, PoolItemState>>>,

    element_size: Entity<f32>,

    bounds: Entity<Option<Bounds<Pixels>>>,

    current_dragging_pool: Entity<Option<usize>>,

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

            current_dragging_pool: cx.new(|_| None),

            _subscriptions,
        }
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

impl Panel for MultiPoolPanel {
    fn panel_name(&self) -> &'static str {
        MULTIPOOL_PANEL_NAME
    }

    fn title(&mut self, _: &mut gpui::Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
        "Multipool"
    }

    fn toolbar_buttons(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> Option<Vec<gpui_component::button::Button>> {
        Some(toolbar_buttons(self, window, cx))
    }

    fn dump(&self, _: &App) -> gpui_component::dock::PanelState {
        let mut state = PanelState::new(self);
        state.info = PanelInfo::Panel(
            serde_json::to_value(self.config.clone()).unwrap_or(serde_json::Value::Null),
        );
        state
    }

    fn inner_padding(&self, _: &App) -> bool {
        false
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

    fn render_grid(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
                cx.listener(|this, _, _, cx| {
                    this.current_dragging_pool.update(cx, |pool, _| {
                        *pool = None;
                    });
                    cx.notify();
                }),
            )
            .on_mouse_move(cx.listener(|this, evt: &MouseMoveEvent, _, cx| {
                if let Some(current_draging_pool) = *this.current_dragging_pool.read(cx) {
                    if let Some(mouse_grid_pos) = this.get_mouse_grid_pos(&evt.position, cx) {
                        this.config.pools[current_draging_pool].resize(mouse_grid_pos);

                        cx.notify();
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
                    .when(
                        self.current_dragging_pool
                            .read(cx)
                            .is_some_and(|dragging_pool| dragging_pool == pool_idx),
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
                                                apply_pool_type_to_button(p.pool_type, this, id)
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
                                    this.current_dragging_pool.update(cx, |pool, _| {
                                        *pool = Some(pool_idx);
                                    });
                                    cx.notify();
                                }),
                            )
                            .child(IconName::ResizeCorner),
                    )
            }))
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
