use gpui::{
    AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement,
    Render, Styled, Subscription, Window, div,
};
use gpui_component::{
    Sizable,
    dock::{Panel, PanelEvent},
    input::{Input, InputEvent, InputState},
};

use crate::{engine::DemexEngineHandler, ui2::config::AppConfigExt};

pub struct CommandPanel {
    focus_handle: FocusHandle,

    command_input_state: Entity<InputState>,

    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<PanelEvent> for CommandPanel {}
impl Focusable for CommandPanel {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for CommandPanel {
    fn panel_name(&self) -> &'static str {
        "command"
    }
}

impl CommandPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let command_input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor("rust")
                .line_number(false)
                .indent_guides(false)
                .rows(1)
                .placeholder("Command")
        });

        let subs = vec![cx.subscribe_in(
            &command_input_state,
            window,
            |_, input, event: &InputEvent, window, cx| match event {
                InputEvent::PressEnter { .. } => {
                    let command = input.read(cx).value();

                    if let Err(err) = DemexEngineHandler::engine(cx).exec_command(&command) {
                        log::warn!("Failed to run command \"{}\": {}", command, err);
                    }

                    input.update(cx, |input, cx| {
                        input.set_value("", window, cx);
                        cx.notify();
                    });
                }
                _ => {}
            },
        )];

        Self {
            focus_handle: cx.focus_handle(),
            command_input_state,
            _subscriptions: subs,
        }
    }
}

impl Render for CommandPanel {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        div().size_full().flex().items_end().p_2().child(
            div().p_1().w_full().border_1().child(
                Input::new(&self.command_input_state)
                    .font_family("JetBrains Mono")
                    .with_size(cx.ui_config().ui_size()),
            ),
        )
    }
}
