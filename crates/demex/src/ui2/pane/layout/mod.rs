use std::ops::Range;

use demex_core::channel3::feature::feature_group::FixtureChannel3FeatureGroup;
use demex_ui::{button::button, container::container};
use gpui::{App, Bounds, Entity, Pixels, Window, div, point, prelude::*, px, uniform_list};

use crate::ui2::pane::layout::{
    element::{
        LayoutViewElement, LayoutViewElementType,
        fixture_sheet::FixtureSheet,
        playback::Playback,
        pool::{Pool, preset::PresetPool},
    },
    page::LayoutViewPage,
};

pub mod element;
pub mod page;

const PAGE_SELECTOR_WIDTH: Pixels = px(100.0);

pub(crate) const GRID_N_COLS: u16 = 15;
pub(crate) const GRID_N_ROWS: u16 = 15;

pub struct LayoutViewPane {
    pages: Vec<Entity<LayoutViewPage>>,
    selected_page: usize,
}

impl LayoutViewPane {
    pub fn new(window: &mut Window, cx: &mut App) -> Self {
        Self {
            pages: vec![
                cx.new(|cx| {
                    let elements = vec![
                        LayoutViewElement {
                            from: point(0, 0),
                            to: point(10, 10),
                            element_type: LayoutViewElementType::FixtureSheet(
                                cx.new(|cx| FixtureSheet::new(window, cx)),
                            ),
                        },
                        LayoutViewElement {
                            from: point(0, 10),
                            to: point(15, 15),
                            element_type: LayoutViewElementType::Playback(
                                cx.new(|cx| Playback::new(window, cx, 1)),
                            ),
                        },
                        LayoutViewElement {
                            from: point(10, 0),
                            to: point(15, 3),
                            element_type: LayoutViewElementType::PresetPool(cx.new(|cx| {
                                Pool::new(
                                    window,
                                    cx,
                                    PresetPool::new(FixtureChannel3FeatureGroup::Dimmer),
                                    Bounds::from_corners(point(10, 0), point(15, 3)),
                                )
                            })),
                        },
                        LayoutViewElement {
                            from: point(10, 4),
                            to: point(15, 7),
                            element_type: LayoutViewElementType::PresetPool(cx.new(|cx| {
                                Pool::new(
                                    window,
                                    cx,
                                    PresetPool::new(FixtureChannel3FeatureGroup::Position),
                                    Bounds::from_corners(point(10, 5), point(15, 7)),
                                )
                            })),
                        },
                    ];

                    LayoutViewPage::new(cx, "Programming", elements)
                }),
                cx.new(|cx| LayoutViewPage::new(cx, "Playback", vec![])),
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
