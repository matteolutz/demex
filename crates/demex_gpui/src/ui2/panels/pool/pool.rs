use std::collections::HashMap;

use demex_core::{
    event::DemexEvent,
    pool::{PoolItem, PoolType},
};
use gpui::{
    App, AppContext, Bounds, Context, Entity, IntoElement, ParentElement, Pixels, Render, Styled,
    Subscription, Window, div, prelude::FluentBuilder, px,
};
use gpui_component::PixelsExt;
use itertools::Itertools;

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::{
        ext::GpuiContextExtension,
        panels::pool::{
            pool_action::{apply_pool_type_to_button, handle_pool_item_click},
            pool_button::{PoolButton, PoolItemButtonIndicatorColor},
            pool_item::{PoolItemNameExt, PoolItemState},
            pool_quick_actions::PoolQuickActionsState,
        },
        utils::bounds,
    },
};

const ELEMENT_PADDING: f32 = 5.0;
const ELEMENT_SIZE: f32 = 80.0;

pub struct Pool {
    bounds: Entity<Option<Bounds<Pixels>>>,

    quick_actions_state: Entity<PoolQuickActionsState>,

    pool_items: Entity<Vec<PoolItem>>,
    pool_type: PoolType,

    pool_item_states: Entity<HashMap<u32, PoolItemState>>,

    _subscriptions: Vec<Subscription>,
}

impl Pool {
    pub fn new(pool_type: PoolType, cx: &mut Context<Self>) -> Self {
        let bounds = cx.new(|_| None);
        let quick_actions_state = cx.new(|_| PoolQuickActionsState::default());

        let pool_items = DemexUiState::pool(pool_type, cx);

        let mut _subscriptions = vec![
            cx.observe_and_notify(&bounds),
            cx.observe_and_notify(&quick_actions_state),
            cx.observe_and_notify(&pool_items),
        ];
        _subscriptions.extend(Self::get_pool_type_subscriptions(pool_type, cx));

        Self {
            pool_items,
            pool_type,
            pool_item_states: cx.new(|_| HashMap::new()),
            bounds,
            quick_actions_state,
            _subscriptions,
        }
    }

    fn clear_states(&self, cx: &mut Context<Self>) {
        self.pool_item_states.update(cx, |states, _| {
            states.clear();
        });
        cx.notify();
    }

    fn clear_and_set_state(&self, id: u32, state: PoolItemState, cx: &mut Context<Self>) {
        self.pool_item_states.update(cx, |states, _| {
            states.clear();
            states.insert(id, state);
        });
        cx.notify();
    }

    fn modify_state(
        &self,
        id: u32,
        f: impl FnOnce(Option<PoolItemState>) -> Option<PoolItemState>,
        cx: &mut Context<Self>,
    ) {
        self.pool_item_states.update(cx, |states, _| {
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
                    |this, selection, cx| {
                        if let Some(group_id) =
                            selection.read(cx).as_ref().and_then(|sel| sel.group_id())
                        {
                            this.clear_and_set_state(
                                group_id,
                                PoolItemState {
                                    indicator_color: PoolItemButtonIndicatorColor::Green,
                                },
                                cx,
                            );
                        } else {
                            this.clear_states(cx);
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
                        |this, _, evt, cx| match evt {
                            DemexEvent::ExecutorGo(id) => {
                                this.modify_state(
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
                                this.modify_state(*id, |_| None, cx);
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
                    |this, selected_sequence, cx| {
                        if let Some(selected_seq) = selected_sequence.read(cx) {
                            this.clear_and_set_state(
                                *selected_seq,
                                PoolItemState {
                                    indicator_color: PoolItemButtonIndicatorColor::Blue,
                                },
                                cx,
                            );
                        } else {
                            this.clear_states(cx);
                        }
                    },
                )]
            }
            _ => vec![],
        }
    }
}

impl Pool {
    fn width(&self, cx: &App) -> Option<Pixels> {
        self.bounds.read(cx).map(|bounds| bounds.size.width)
    }

    fn cols_and_element_size(&self, width: Pixels) -> (u16, f32) {
        let n_cols = (width.as_f32() / ELEMENT_SIZE) as u16;
        let element_size = (width.as_f32() / n_cols as f32) - ELEMENT_PADDING;
        (n_cols, element_size)
    }

    fn render_grid(&mut self, _window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(width) = self.width(cx) else {
            return div().absolute().top_0().left_0().size_full();
        };

        let (n_cols, element_size) = self.cols_and_element_size(width);
        let pool_items = self.pool_items.read(cx);

        div()
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .grid()
            .grid_cols(n_cols)
            .gap_y(px(ELEMENT_PADDING))
            .children(pool_items.into_iter().sorted().map(|item| {
                PoolButton::new(item.id as usize)
                    .size(px(element_size))
                    .quick_actions_state(&self.quick_actions_state)
                    .when(true, |this| {
                        apply_pool_type_to_button(self.pool_type, this, Some(item), item.id)
                    })
                    .item_name(item.name.clone().to_name(cx))
                    .on_click({
                        let id = item.id;
                        cx.listener(move |this, _, _, cx| {
                            handle_pool_item_click(this.pool_type, id, cx)
                        })
                    })
                    .when_some(
                        self.pool_item_states.read(cx).get(&item.id),
                        |this, state| this.indicator_color(state.indicator_color),
                    )
                    .when_some(item.colors.as_ref(), |this, colors| {
                        this.colors_rgb(colors.iter().copied())
                    })
                    .item_id(item.id)
            }))
    }
}

impl Render for Pool {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div().size_full().p_8().child(
            div()
                .size_full()
                .relative()
                .child(bounds(&self.bounds).absolute().top_0().left_0().size_full())
                .child(self.render_grid(window, cx)), /*
                                                      .when_some(
                                                          self.bounds.read(cx).clone(),
                                                          |this, bounds: Bounds<Pixels>| {
                                                              this.child(PoolQuickActions::new(&self.quick_actions_state, bounds))
                                                          },
                                                      ),*/
        )
    }
}
