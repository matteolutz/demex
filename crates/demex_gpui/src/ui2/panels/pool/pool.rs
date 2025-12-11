use demex_core::pool::{PoolItem, PoolType};
use gpui::{
    App, AppContext, Bounds, Context, Entity, IntoElement, ParentElement, Pixels, Render, Styled,
    Subscription, Window, div, prelude::FluentBuilder, px,
};
use gpui_component::PixelsExt;
use itertools::Itertools;

use crate::{
    engine::state::DemexUiState,
    ui2::{
        ext::GpuiContextExtension,
        panels::pool::{
            pool_action::handle_pool_item_click,
            pool_button::{PoolButton, PoolItemButtonIndicatorColor},
            pool_item::PoolItemNameExt,
            pool_quick_actions::{PoolQuickActions, PoolQuickActionsState},
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
            bounds,
            quick_actions_state,
            _subscriptions,
        }
    }

    fn get_pool_type_subscriptions(
        pool_type: PoolType,
        cx: &mut Context<Self>,
    ) -> Vec<Subscription> {
        match pool_type {
            PoolType::Group => vec![cx.observe_and_notify(&DemexUiState::fixture_selection(cx))],
            _ => vec![],
        }
    }

    fn get_pool_item_color(
        &self,
        pool_item_id: u32,
        cx: &App,
    ) -> Option<PoolItemButtonIndicatorColor> {
        match self.pool_type {
            PoolType::Group => {
                let is_selected = DemexUiState::fixture_selection(cx)
                    .read(cx)
                    .as_ref()
                    .and_then(|sel| sel.group_id())
                    .is_some_and(|group_id| group_id == pool_item_id);
                is_selected.then_some(PoolItemButtonIndicatorColor::Green)
            }
            _ => None,
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
                    .action("Test", |_, _| log::debug!("PoolButton: Test"))
                    .action("Test 2", |_, _| log::debug!("PoolButton: Test 2"))
                    .action("Test 3", |_, _| log::debug!("PoolButton: Test 3"))
                    .action("Test 4", |_, _| log::debug!("PoolButton: Test 4"))
                    .action("Test 5", |_, _| log::debug!("PoolButton: Test 5"))
                    .action("Test 6", |_, _| log::debug!("PoolButton: Test 6"))
                    .action("Test 7", |_, _| log::debug!("PoolButton: Test 7"))
                    .action("Test 8", |_, _| log::debug!("PoolButton: Test 8"))
                    .item_name(item.name.clone().to_name(cx))
                    .on_click({
                        let id = item.id;
                        cx.listener(move |this, _, _, cx| {
                            handle_pool_item_click(this.pool_type, id, cx)
                        })
                    })
                    .when_some(self.get_pool_item_color(item.id, cx), |this, color| {
                        this.indicator_color(color)
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
                .child(self.render_grid(window, cx))
                .when_some(
                    self.bounds.read(cx).clone(),
                    |this, bounds: Bounds<Pixels>| {
                        let width = bounds.size.width;
                        let (_, element_size) = self.cols_and_element_size(width);

                        this.child(PoolQuickActions::new(
                            &self.quick_actions_state,
                            bounds,
                            element_size,
                            ELEMENT_PADDING,
                        ))
                    },
                ),
        )
    }
}
