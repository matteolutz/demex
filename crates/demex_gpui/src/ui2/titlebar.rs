use gpui::{ParentElement, Render, Styled, div};
use gpui_component::{
    IconName, TitleBar,
    button::{Button, ButtonVariants},
};

#[derive(Default)]
pub struct DemexTitleBar {}

impl Render for DemexTitleBar {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        TitleBar::new().child(
            div()
                .flex()
                .items_center()
                .justify_end()
                .gap_2()
                .child("demex")
                .child(Button::new("save").ghost().icon(IconName::Copy)),
        )
    }
}
