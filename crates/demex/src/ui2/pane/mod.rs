use demex_ui::container::container;
use demex_ui::input::{TextInput, TextInputEvent};
use demex_ui::theme::ActiveTheme;
use gpui::{Context, Entity, Render, Styled, div};
use gpui::{prelude::*, px};

use crate::engine::DemexEngineHandler;
use crate::ui2::pane::layout::LayoutViewPane;

pub mod layout;

pub struct MainPane {
    layout_pane: Entity<LayoutViewPane>,
    command_input: Entity<TextInput>,
}

impl MainPane {
    pub fn new(window: &mut gpui::Window, cx: &mut Context<Self>) -> Self {
        let command_input = cx.new(|cx| {
            let t = TextInput::new("command", cx.focus_handle(), window, cx);
            // t.set_placeholder("Enter command".into(), cx);
            t
        });

        cx.subscribe_in(
            &command_input,
            window,
            |_, input, event, _, cx| match event {
                TextInputEvent::Submit(command) => {
                    if let Err(err) = DemexEngineHandler::engine(cx).exec_command(command) {
                        log::warn!("Failed to run command \"{}\": {}", command, err);
                    }

                    input.update(cx, |input, cx| {
                        input.set_text("".into(), cx);
                        cx.notify();
                    });
                }
                _ => {}
            },
        )
        .detach();

        Self {
            layout_pane: cx.new(|cx| LayoutViewPane::new(window, cx)),
            command_input,
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
                    .h(px(50.0))
                    .flex()
                    .items_center()
                    .p_2()
                    .child(
                        div()
                            .p_1()
                            .w_full()
                            .border_1()
                            .border_color(cx.theme().border)
                            .child(self.command_input.clone()),
                    ),
            )
    }
}
