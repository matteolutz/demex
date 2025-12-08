use gpui::{
    App, AppContext, Bounds, Context, Entity, IntoElement, ParentElement, Pixels, Render,
    SharedString, Styled, Subscription, Window, div, prelude::FluentBuilder, px,
};
use gpui_component::PixelsExt;

use crate::ui2::{
    ext::GpuiContextExtension,
    panels::pool::{
        pool_button::{PoolButton, PoolItemButtonIndicatorColor},
        pool_quick_actions::{PoolQuickActions, PoolQuickActionsState},
    },
    utils::bounds,
};

const ELEMENT_PADDING: f32 = 5.0;
const ELEMENT_SIZE: f32 = 80.0;

#[derive(Debug, Clone)]
pub struct PoolItem {
    id: u32,
    name: SharedString,
}

pub struct Pool {
    items: Vec<PoolItem>,
    bounds: Entity<Option<Bounds<Pixels>>>,

    quick_actions_state: Entity<PoolQuickActionsState>,

    _subscriptions: Vec<Subscription>,
}

impl Pool {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let bounds = cx.new(|_| None);
        let quick_actions_state = cx.new(|_| PoolQuickActionsState::default());

        let _subscriptions = vec![
            cx.observe_and_notify(&bounds),
            cx.observe_and_notify(&quick_actions_state),
        ];

        Self {
            items: (0..10)
                .map(|id| PoolItem {
                    id,
                    name: format!("Item {}", id + 1).into(),
                })
                .collect(),
            bounds,
            quick_actions_state,
            _subscriptions,
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

        div()
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .grid()
            .grid_cols(n_cols)
            .gap_y(px(ELEMENT_PADDING))
            .children(self.items.iter().enumerate().map(|(idx, item)| {
                PoolButton::new(item.id as usize)
                    .size(px(element_size))
                    .indicator_color(match idx % 3 {
                        0 => PoolItemButtonIndicatorColor::Red,
                        1 => PoolItemButtonIndicatorColor::Green,
                        2 => PoolItemButtonIndicatorColor::Blue,
                        _ => unreachable!(),
                    })
                    .quick_actions_state(&self.quick_actions_state)
                    .action("Test", |_, _| println!("Test"))
                    .action("Test 2", |_, _| println!("Test 2"))
                    .action("Test 3", |_, _| println!("Test 3"))
                    .action("Test 4", |_, _| println!("Test 4"))
                    .action("Test 5", |_, _| println!("Test 5"))
                    .action("Test 6", |_, _| println!("Test 6"))
                    .action("Test 7", |_, _| println!("Test 7"))
                    .action("Test 8", |_, _| println!("Test 8"))
                    .on_click(|_, _, _| println!("clicked"))
                    .item_name(&item.name)
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
