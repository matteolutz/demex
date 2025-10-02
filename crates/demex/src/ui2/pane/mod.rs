use demex_ui::container::container;
use demex_ui::input::TextInput;
use gpui::{Context, Entity, Render, Styled, div};
use gpui::{prelude::*, px};

use crate::ui2::pane::layout::LayoutViewPane;

pub mod layout;

pub struct MainPane {
    layout_pane: Entity<LayoutViewPane>,
    command_input: Entity<TextInput>,
}

impl MainPane {
    pub fn new(window: &mut gpui::Window, cx: &mut Context<Self>) -> Self {
        Self {
            layout_pane: cx.new(|cx| LayoutViewPane::new(window, cx)),
            command_input: cx.new(|cx| TextInput::new("command", cx.focus_handle(), window, cx)),
        }
    }
}

impl Render for MainPane {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                container(window, cx)
                    .w_full()
                    .flex_1()
                    .child(self.layout_pane.clone()),
            )
            .child(
                container(window, cx)
                    .w_full()
                    .h(px(100.0))
                    .child(div().child(self.command_input.clone())),
            )
    }
}
