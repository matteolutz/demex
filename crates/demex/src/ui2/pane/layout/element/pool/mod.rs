use demex_ui::{container::interactive_container, theme::ActiveTheme};
use gpui::prelude::*;
use gpui::{Bounds, ParentElement, Render, Styled, Window, div};

use crate::ui2::pane::layout::element::pool::delegate::PoolDelegate;

pub mod delegate;
pub mod group;
pub mod preset;

pub struct Pool<P: PoolDelegate> {
    pool: P,
    bounds: Bounds<u16>,
}

impl<P: PoolDelegate> Pool<P> {
    pub fn new(pool: P, bounds: Bounds<u16>) -> Self {
        Self { pool, bounds }
    }
}

impl<P: PoolDelegate> Render for Pool<P> {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let n_entries = ((self.bounds.size.width * self.bounds.size.height) - 1) as usize;
        let pool_ids = self.pool.get_ids(cx, 0..n_entries);

        div()
            .size_full()
            .grid()
            .grid_cols(self.bounds.size.width)
            .grid_rows(self.bounds.size.height)
            .child(
                div()
                    .col_span(1)
                    .row_span(1)
                    .bg(cx.theme().accent)
                    .child(self.pool.get_title()),
            )
            .children(pool_ids.into_iter().enumerate().map(|(index, id)| {
                interactive_container(index, None)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.pool.on_action(id, P::PoolItemAction::default(), cx);
                    }))
                    .col_span(1)
                    .row_span(1)
                    .child(if let Some(data) = self.pool.get_item_data(id, cx) {
                        data.name
                    } else {
                        "".to_string()
                    })
            }))
    }
}
