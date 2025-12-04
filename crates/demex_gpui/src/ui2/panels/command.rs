use gpui::{
    App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, Styled, Subscription, Window, div,
};
use gpui_component::{
    Sizable,
    dock::{Panel, PanelEvent, register_panel},
    input::{Input, InputEvent, InputState},
    notification::Notification,
};

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::{config::AppConfigExt, wm::app::WindowManagerAppExt},
};

const COMMAND_PANEL_NAME: &str = "demex-command";

pub(super) fn register(cx: &mut App) {
    register_panel(cx, COMMAND_PANEL_NAME, |_, _, _, window, cx| {
        Box::new(cx.new(|cx| CommandPanel::new(window, cx)))
    });
}

pub struct CommandPanel {
    focus_handle: FocusHandle,

    command_input_state: Entity<InputState>,

    /// Indexed from the back
    command_history_idx: Entity<Option<usize>>,

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
        COMMAND_PANEL_NAME
    }
}

impl CommandPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let command_input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor("demex")
                .line_number(false)
                .indent_guides(false)
                .rows(1)
                .placeholder("Command")
        });

        let command_history_idx = cx.new(|_| None);

        let subs = vec![
            cx.subscribe_in(
                &command_input_state,
                window,
                |_, input, event: &InputEvent, window, cx| match event {
                    InputEvent::PressEnter { .. } => {
                        let command = input.read(cx).value();

                        if let Err(err) = DemexEngineHandler::engine(cx).exec_command(&command) {
                            log::warn!("Failed to run command \"{}\": {}", command, err);
                            cx.update_wm(|wm, cx| {
                                wm.push_notifcation(Notification::error(err.to_string()), cx)
                            });
                        }

                        input.update(cx, |input, cx| {
                            input.set_value("", window, cx);
                            cx.notify();
                        });
                    }
                    _ => {}
                },
            ),
            cx.observe_in(
                &command_history_idx,
                window,
                |this, idx_entity, window, cx| {
                    let Some(idx) = *idx_entity.read(cx) else {
                        return;
                    };

                    let value = DemexUiState::command_history(cx).read_with(cx, |history, _| {
                        history.get(history.len() - idx - 1).cloned()
                    });

                    let Some(value) = value else {
                        return;
                    };

                    this.command_input_state.update(cx, |input, cx| {
                        input.set_value(&value, window, cx);
                        cx.notify();
                    });

                    cx.notify();
                },
            ),
        ];

        Self {
            focus_handle: cx.focus_handle(),
            command_history_idx,
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
