use std::ops::Range;

use demex_ui::{button::button, container::container};
use gpui::{App, Entity, Pixels, div, prelude::*, px, rgb, uniform_list};

use crate::ui2::pane::layout::page::LayoutViewPage;

pub mod element;
pub mod page;

const PAGE_SELECTOR_WIDTH: Pixels = px(100.0);

pub struct LayoutViewPane {
    pages: Vec<Entity<LayoutViewPage>>,
    selected_page: usize,
}

impl LayoutViewPane {
    pub fn new(cx: &mut App) -> Self {
        Self {
            pages: vec![
                cx.new(|_| LayoutViewPage::new("Programming", vec![])),
                cx.new(|_| LayoutViewPage::new("Playback", vec![])),
            ],
            selected_page: 0,
        }
    }
}

impl LayoutViewPane {
    fn render_layout_elements(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        container(window, cx)
            .h_full()
            .flex_1()
            .child(self.pages[self.selected_page].clone())
    }

    fn render_layout_page_list(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        container(window, cx)
            .h_full()
            .w(PAGE_SELECTOR_WIDTH)
            .flex()
            .flex_col()
            .child(
                uniform_list(
                    "LayoutViewPane",
                    self.pages.len().min(10),
                    cx.processor(|this, range: Range<usize>, _, cx| {
                        range
                            .map(|idx| {
                                button(idx, None, this.pages[idx].read(cx).name().to_string())
                                    .size(PAGE_SELECTOR_WIDTH)
                                    .selected(this.selected_page == idx)
                                    .on_click(cx.listener(move |this, _, _, _| {
                                        this.selected_page = idx;
                                    }))
                            })
                            .collect::<Vec<_>>()
                    }),
                )
                .size_full(),
            )
    }
}

impl Render for LayoutViewPane {
    fn render(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_row()
            .child(self.render_layout_elements(window, cx))
            .child(self.render_layout_page_list(window, cx))
    }
}
