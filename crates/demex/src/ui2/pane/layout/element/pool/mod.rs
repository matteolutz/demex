use demex_ui::theme::ActiveTheme;
use demex_ui::utils::todo;
use gpui::{Bounds, ParentElement, Render, Styled, Window, div};

use crate::ui2::pane::layout::element::pool::delegate::PoolDelegate;

pub mod delegate;
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
            .children(self.pool.get_ids(cx, 0..n_entries).iter().map(|id| {
                if let Some(data) = self.pool.get_item_data(id, cx) {
                    div().col_span(1).row_span(1).child(data.name)
                } else {
                    todo(cx).col_span(1).row_span(1)
                }
            }))
    }
}
