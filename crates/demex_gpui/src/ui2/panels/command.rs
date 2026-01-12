use gpui::{
    App, AppContext, BoxShadow, Context, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement, IntoElement, ParentElement, Render, Styled, Subscription,
    UniformListScrollHandle, Window, div, point, prelude::FluentBuilder, uniform_list,
};
use gpui_component::{
    ActiveTheme, Icon, IconName, Sizable,
    button::Button,
    dock::{Panel, PanelEvent, register_panel},
    h_flex,
    input::{Input, InputEvent, InputState, Position},
    notification::Notification,
    scroll::ScrollableElement,
    v_flex,
};

use crate::{
    engine::{
        DemexEngineHandler,
        state::{DemexCommandHistoryEntry, DemexUiState},
    },
    ui2::{config::AppConfigExt, panels::toolbar_buttons, wm::app::WindowManagerAppExt},
};

const COMMAND_PANEL_NAME: &str = "demex-command";

pub(super) fn register(cx: &mut App) {
    register_panel(cx, COMMAND_PANEL_NAME, |_, _, _, window, cx| {
        Box::new(cx.new(|cx| CommandPanel::new(window, cx)))
    });
}

mod actions {
    use gpui::{App, KeyBinding, actions};

    pub const CONTEXT: &str = "demex-command-input";
    actions!(command_input, [PrevCommand, NextCommand]);

    pub fn init(cx: &mut App) {
        cx.bind_keys([
            KeyBinding::new("secondary-up", PrevCommand, Some(CONTEXT)),
            KeyBinding::new("secondary-down", NextCommand, Some(CONTEXT)),
        ]);
    }
}

pub fn init(cx: &mut App) {
    actions::init(cx);
}

pub struct CommandPanel {
    focus_handle: FocusHandle,

    command_input_state: Entity<InputState>,

    // Indexed from the back
    command_history_idx: Entity<Option<usize>>,

    command_history_scroll_handle: UniformListScrollHandle,

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

    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        "Command"
    }

    fn toolbar_buttons(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Vec<Button>> {
        Some(toolbar_buttons(self, window, cx))
    }
}

impl CommandPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let command_input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor("demex")
                .line_number(false)
                .indent_guides(false)
                .placeholder("Command")
        });

        let command_history_idx = cx.new(|_| None);

        let subs = vec![
            cx.subscribe_in(
                &command_input_state,
                window,
                |this, input, event: &InputEvent, window, cx| match event {
                    InputEvent::PressEnter { .. } => {
                        this.handle_command_input_submit(input, window, cx)
                    }
                    _ => {}
                },
            ),
            cx.observe_in(
                &command_history_idx,
                window,
                Self::handle_command_history_idx_update,
            ),
        ];

        Self {
            focus_handle: cx.focus_handle(),
            command_history_idx,
            command_input_state,
            command_history_scroll_handle: UniformListScrollHandle::new(),
            _subscriptions: subs,
        }
    }
}

impl CommandPanel {
    fn handle_command_input_submit(
        &mut self,
        input_state: &Entity<InputState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let input_value = input_state.read(cx).value();
        let command = input_value.strip_suffix("\n");

        let Some(command) = command else {
            return;
        };

        if !command.is_empty() {
            let exec_res = DemexEngineHandler::engine(cx).exec_command(command);
            let is_success = exec_res.is_ok();

            if let Err(err) = exec_res {
                log::warn!("Failed to run command \"{}\": {}", command, err);
                cx.update_wm(|wm, cx| {
                    wm.push_notifcation(Notification::error(err.to_string()), cx)
                });
            }

            let history_length = DemexUiState::command_history(cx).update(cx, |history, cx| {
                history.push_now(command.into(), is_success);
                cx.notify();
                history.len()
            });

            if history_length > 0 {
                self.command_history_scroll_handle
                    .scroll_to_item(history_length - 1, gpui::ScrollStrategy::Bottom);
            }

            // don't notify
            self.command_history_idx.update(cx, |idx, _| *idx = None);
        }

        input_state.update(cx, |input, cx| {
            input.set_value("", window, cx);
            cx.notify();
        });

        cx.notify();
    }

    fn handle_command_history_idx_update(
        &mut self,
        idx_entity: Entity<Option<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let command_history_len = DemexUiState::command_history(cx).read(cx).len();

        let Some(idx) = *idx_entity.read(cx) else {
            // When idx was set to None, clear the command input state
            self.command_input_state
                .update(cx, |input, cx| input.set_value("", window, cx));
            self.command_history_scroll_handle
                .scroll_to_item(command_history_len - 1, gpui::ScrollStrategy::Bottom);
            return;
        };

        let value: Option<DemexCommandHistoryEntry> =
            DemexUiState::command_history(cx).read_with(cx, |history, _| history.get(idx).cloned());

        let Some(value) = value else {
            return;
        };

        self.command_input_state.update(cx, |input, cx| {
            input.set_value(&value.command, window, cx);
            input.set_cursor_position(Position::new(0, value.command.len() as u32), window, cx);
        });

        let rev_idx = command_history_len - idx - 1;
        self.command_history_scroll_handle
            .scroll_to_item(rev_idx, gpui::ScrollStrategy::Bottom);

        cx.notify();
    }

    fn handle_prev_command(
        &mut self,
        _: &actions::PrevCommand,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let command_history_len = DemexUiState::command_history(cx).read(cx).len();

        self.command_history_idx.update(cx, |idx, cx| {
            if command_history_len == 0 {
                *idx = None;
            } else {
                match idx {
                    Some(idx) => *idx = (*idx + 1).min(command_history_len - 1),
                    None => *idx = Some(0),
                }
            }
            cx.notify();
        });
    }

    fn handle_next_command(
        &mut self,
        _: &actions::NextCommand,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.command_history_idx.update(cx, |idx, cx| {
            match idx {
                Some(0) => *idx = None,
                Some(idx) => *idx -= 1,
                _ => {}
            }
            cx.notify();
        });
    }
}

impl CommandPanel {
    // TODO: move into it's own component
    fn render_command_history(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let command_history = DemexUiState::command_history(cx);
        let selected_history_item = self.command_history_idx.read(cx).clone();

        div()
            .relative()
            .w_full()
            .flex_1()
            .overflow_hidden()
            .child(
                v_flex()
                    .absolute()
                    .top_0()
                    .left_0()
                    .flex_col_reverse()
                    .rounded(cx.theme().radius)
                    .size_full()
                    .justify_end()
                    .bg(cx.theme().group_box)
                    .gap_0()
                    .vertical_scrollbar(&self.command_history_scroll_handle)
                    .child(
                        uniform_list(
                            "command_history",
                            command_history.read(cx).iter().count(),
                            move |visible_range, _, cx| {
                                let command_history_len = command_history.read(cx).len();

                                command_history
                                    .read(cx)
                                    .iter()
                                    .enumerate()
                                    .skip(visible_range.start)
                                    .take(visible_range.end - visible_range.start)
                                    .map(|(idx, command)| {
                                        h_flex()
                                            .font_family("JetBrains Mono")
                                            .border_t_1()
                                            .border_color(cx.theme().border)
                                            .px_3()
                                            .py_1()
                                            .gap_2()
                                            .text_sm()
                                            .w_full()
                                            .when(
                                                selected_history_item.is_some_and(|sel_idx| {
                                                    (command_history_len - sel_idx - 1) == idx
                                                }),
                                                |this| this.underline(),
                                            )
                                            .child(
                                                Icon::new(IconName::ChevronRight)
                                                    .when(!command.success, |icon| {
                                                        icon.text_color(cx.theme().red)
                                                    })
                                                    .when(command.success, |icon| {
                                                        icon.text_color(cx.theme().green)
                                                    }),
                                            )
                                            .child(
                                                div()
                                                    .child(
                                                        command
                                                            .timestamp
                                                            .format("%H:%M:%S")
                                                            .to_string(),
                                                    )
                                                    .text_color(cx.theme().muted_foreground),
                                            )
                                            .child(command.command.clone())
                                    })
                                    .collect()
                            },
                        )
                        .size_full()
                        .y_flipped(false)
                        .track_scroll(&self.command_history_scroll_handle),
                    ),
            )
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .w_full()
                    .h_0()
                    .shadow(vec![BoxShadow {
                        color: cx.theme().background,
                        offset: point((0.0).into(), (10.0).into()),
                        blur_radius: (40.0).into(),
                        spread_radius: (20.0).into(),
                    }]),
            )
    }
}

impl Render for CommandPanel {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .size_full()
            .justify_end()
            .p_3()
            .gap_2()
            .child(self.render_command_history(window, cx))
            .child(
                div()
                    .key_context(actions::CONTEXT)
                    .on_action(cx.listener(Self::handle_prev_command))
                    .on_action(cx.listener(Self::handle_next_command))
                    .w_full()
                    .border_1()
                    .child(
                        Input::new(&self.command_input_state)
                            .font_family("JetBrains Mono")
                            .with_size(cx.ui_config().ui_size()),
                    ),
            )
    }
}
