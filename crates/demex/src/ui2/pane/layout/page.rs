use gpui::prelude::*;

use demex_ui::{
    grid::{dot_grid, dot_grid_fixed},
    theme::ActiveTheme,
};
use gpui::{Render, px};

use crate::ui2::pane::layout::element::LayoutViewElement;

const GRID_N_COLS: usize = 15;
const GRID_N_ROWS: usize = 15;

pub struct LayoutViewPage {
    name: String,
    elements: Vec<LayoutViewElement>,
}

impl LayoutViewPage {
    pub fn new(name: impl Into<String>, elements: Vec<LayoutViewElement>) -> Self {
        Self {
            name: name.into(),
            elements,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Render for LayoutViewPage {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        dot_grid_fixed(GRID_N_COLS, GRID_N_ROWS, cx.theme().accent)
            .size_full()
            .mx(px(GRID_N_COLS as f32 / 1.0))
            .my(px(GRID_N_ROWS as f32 / 1.0))
    }
}
