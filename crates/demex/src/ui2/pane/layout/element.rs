use demex_ui::utils::todo;
use gpui::{Entity, Point, Render};
use strum::EnumIter;

#[derive(EnumIter)]
pub enum LayoutViewElementType {
    FixtureSheet,
}

pub struct LayoutViewElement {
    pub(crate) from: Point<u16>,
    pub(crate) to: Point<u16>,
    pub(crate) element_type: Entity<LayoutViewElementType>,
}

impl Render for LayoutViewElementType {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        todo(cx)
    }
}
