use std::collections::HashMap;

use demex_core::pool::PoolType;
use gpui::{
    App, AppContext, BorderStyle, Bounds, Context, Entity, EventEmitter, FocusHandle, Focusable,
    Hsla, IntoElement, PaintQuad, ParentElement, Pixels, Render, Styled, Subscription, Window,
    canvas, div, point, prelude::FluentBuilder, px, size,
};
use gpui_component::{
    ActiveTheme, Colorize, Disableable, IconName, PixelsExt, StyledExt,
    button::{Button, ButtonVariants},
    dock::{Panel, PanelEvent, PanelInfo, PanelState, register_panel},
    v_flex,
};

use crate::{
    engine::state::DemexUiState,
    ui2::{
        ext::GpuiContextExtension,
        panels::{
            multipool::config::MultiPoolConfig,
            pool::{
                pool_action::apply_pool_type_to_button,
                pool_button::PoolButton,
                pool_item::{PoolItemNameExt, PoolItemState},
                pool_quick_actions::{PoolQuickActions, PoolQuickActionsState},
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

pub struct MultiPoolPanel {
    focus_handle: FocusHandle,

    config: MultiPoolConfig,

    quick_actions_state: Entity<PoolQuickActionsState>,

    pool_item_states: Entity<HashMap<(PoolType, u32), PoolItemState>>,

    element_size: Entity<f32>,

    bounds: Entity<Option<Bounds<Pixels>>>,

    _subscriptions: Vec<Subscription>,
}

impl MultiPoolPanel {
    pub fn new(config: MultiPoolConfig, cx: &mut Context<Self>) -> Self {
        let quick_actions_state = cx.new(|_| PoolQuickActionsState::default());

        let _subscriptions = vec![cx.observe_and_notify(&quick_actions_state)];

        Self {
            focus_handle: cx.focus_handle(),

            config,
            element_size: cx.new(|_| ELEMENT_SIZE),

            quick_actions_state,

            pool_item_states: cx.new(|_| HashMap::new()),

            bounds: cx.new(|_| None),

            _subscriptions,
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
                            background: cx.theme().red.into(),
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
            .children(self.config.pools.iter().enumerate().map(|(pool_idx, p)| {
                let pool = DemexUiState::pool(p.pool_type, cx).read(cx);

                div()
                    .col_start(p.start_cell.0 as i16 + 1)
                    .col_end(p.start_cell.0 as i16 + 1 + p.size.0 as i16)
                    .row_start(p.start_cell.1 as i16 + 1)
                    .row_end(p.start_cell.1 as i16 + 1 + p.size.1 as i16)
                    .relative()
                    .child(
                        div()
                            .size_full()
                            .grid_cols(p.size.0)
                            .grid_rows(p.size.1)
                            .grid()
                            .bg(cx.theme().accent.blend(Hsla::black().alpha(0.2)))
                            .gap(px(ELEMENT_PADDING))
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
                                    .child(p.pool_type.to_string()),
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
                                            })
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
                            .child(
                                Button::new(("resize", pool_idx))
                                    .icon(IconName::ResizeCorner)
                                    .text()
                                    .cursor_nwse_resize(),
                            ),
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
            .child(bounds(&self.bounds))
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
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.element_size.update(cx, |size, _| *size -= 1.0);
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("zoom-in")
                            .icon(IconName::Plus)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.element_size.update(cx, |size, _| *size += 1.0);
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .size_full()
                    .relative()
                    .child(self.render_dot_grid(window, cx))
                    .child(self.render_grid(window, cx)),
            )
            .when_some(self.bounds.read(cx).clone(), |this, bounds| {
                this.child(PoolQuickActions::new(&self.quick_actions_state, bounds))
            })
    }
}
