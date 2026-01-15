use std::str::FromStr;

use demex_core::{
    command::parser::nodes::action::{Action, functions::set_function::CueSetTriggerArgs},
    engine::comm::SequenceCueRequest,
    sequence::cue::{CueIdx, CueTrigger},
};
use gpui::{
    App, AppContext, Context, Entity, InteractiveElement, ParentElement, Render, SharedString,
    Styled, Subscription, Window, WindowBounds, prelude::FluentBuilder, size,
};
use gpui_component::{
    input::{InputState, NumberInput},
    select::{Select, SelectEvent, SelectItem, SelectState},
    v_flex,
};
use strum::IntoEnumIterator;

use crate::{engine::DemexEngineHandler, ui2::wm::edit_window::EditWindowDelegate};

mod actions {
    use gpui::{App, KeyBinding};

    pub const CONTEXT: &str = "demex-edit-cue-trigger-window";

    gpui::actions!([QuitEditCueTrigger, SubmitEditCueTrigger]);

    pub fn init(cx: &mut App) {
        cx.bind_keys([
            KeyBinding::new("escape", QuitEditCueTrigger, Some(CONTEXT)),
            KeyBinding::new("enter", SubmitEditCueTrigger, Some(CONTEXT)),
        ]);
    }
}

pub(super) fn init(cx: &mut App) {
    actions::init(cx);
}

#[derive(Debug, Clone)]
pub struct CueTriggerItem {
    trigger: CueTrigger,
    name: SharedString,
}

impl From<CueTrigger> for CueTriggerItem {
    fn from(value: CueTrigger) -> Self {
        Self {
            name: value.to_string().into(),
            trigger: value,
        }
    }
}

impl SelectItem for CueTriggerItem {
    type Value = CueTrigger;

    fn title(&self) -> gpui::SharedString {
        self.name.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.trigger
    }
}

// TODO: improve this
pub struct EditCueTriggerWindow {
    sequence_id: u32,
    cue_idx: CueIdx,

    select_state: Entity<SelectState<Vec<SharedString>>>,
    input_state: Entity<InputState>,

    _subscriptions: Vec<Subscription>,
}

impl EditCueTriggerWindow {
    pub fn new(
        sequence_id: u32,
        cue_idx: CueIdx,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let select_state = cx.new(|cx| {
            SelectState::new(
                CueTrigger::iter()
                    .map(|trigger| trigger.to_string().into())
                    .collect(),
                None,
                window,
                cx,
            )
        });

        let input_state = cx.new(|cx| InputState::new(window, cx));

        let _subscriptions = vec![cx.subscribe_in(
            &select_state,
            window,
            move |this, _, evt: &SelectEvent<Vec<SharedString>>, window, cx| match evt {
                SelectEvent::Confirm(trigger) => this.update_text_input_state(
                    trigger.as_ref().and_then(|t| CueTrigger::from_str(t).ok()),
                    window,
                    cx,
                ),
            },
        )];

        DemexEngineHandler::send_in_visual(
            window,
            cx,
            SequenceCueRequest {
                sequence_id,
                cue_idx,
            },
            {
                let select_state = select_state.clone();
                move |this, cue, window, cx| {
                    if let Some(cue) = cue {
                        select_state.update(cx, |state, cx| {
                            state.set_selected_value(&cue.trigger.to_string().into(), window, cx);
                        });
                        this.update_text_input_state(Some(cue.trigger), window, cx);
                    }
                }
            },
        );

        Self {
            sequence_id,
            cue_idx,

            select_state,
            input_state,

            _subscriptions,
        }
    }

    fn update_text_input_state(
        &self,
        trigger: Option<CueTrigger>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match trigger {
            Some(CueTrigger::Time(time)) => {
                self.input_state.update(cx, |state, cx| {
                    state.set_value(time.to_string(), window, cx);
                });
                cx.notify();
            }
            _ => {}
        }
    }

    fn submit(&self, cx: &mut Context<Self>) {
        let Some(mut trigger) = self
            .select_state
            .read(cx)
            .selected_value()
            .cloned()
            .and_then(|trigger| CueTrigger::from_str(trigger.as_str()).ok())
        else {
            return;
        };

        match &mut trigger {
            &mut CueTrigger::Time(ref mut time) => {
                let input_value = self.input_state.read(cx).value();
                log::debug!("input value is: {:?}", input_value);

                *time = input_value.parse().unwrap_or_default();
            }
            _ => {}
        }

        DemexEngineHandler::engine(cx).exec_ui(Action::CueSetTrigger(CueSetTriggerArgs {
            sequence_id: self.sequence_id,
            cue_idx: self.cue_idx,
            trigger: trigger,
        }));

        self.close(cx);
    }
}

impl EditWindowDelegate for EditCueTriggerWindow {
    fn window_title(
        &self,
        _window: &mut gpui::Window,
        _cx: &gpui::App,
    ) -> impl Into<gpui::SharedString> {
        format!("Seq {} Cue {} Trigger", self.sequence_id, self.cue_idx)
    }

    fn window_bounds(cx: &mut App) -> Option<gpui::WindowBounds> {
        Some(WindowBounds::centered(size(500.0.into(), 200.0.into()), cx))
    }

    fn handle_save(&self, _window: &mut gpui::Window, _cx: &mut gpui::App) {}

    fn handle_discard(&self, _window: &mut gpui::Window, _cx: &mut gpui::App) {}
}

impl Render for EditCueTriggerWindow {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        v_flex()
            .key_context(actions::CONTEXT)
            .on_action(cx.listener(|this, _: &actions::QuitEditCueTrigger, _, cx| {
                this.close(cx);
            }))
            .on_action(
                cx.listener(|this, _: &actions::SubmitEditCueTrigger, _, cx| {
                    this.submit(cx);
                }),
            )
            .justify_center()
            .p_4()
            .gap_2()
            .child(Select::new(&self.select_state).w_full())
            .when(
                self.select_state
                    .read(cx)
                    .selected_value()
                    .is_some_and(|val| val == "Time"),
                |this| this.child(NumberInput::new(&self.input_state).suffix("s").w_full()),
            )
    }
}
