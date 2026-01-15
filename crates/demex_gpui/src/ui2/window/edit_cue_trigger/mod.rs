use demex_core::{
    engine::comm::SequenceCueRequest,
    sequence::cue::{CueIdx, CueTrigger},
};
use gpui::{
    App, AppContext, Context, Entity, InteractiveElement, ParentElement, Render, SharedString,
    Styled, Subscription, Window, WindowBounds, prelude::FluentBuilder, size,
};
use gpui_component::{
    input::{InputState, NumberInput},
    select::{Select, SelectItem, SelectState},
    v_flex,
};
use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::{engine::DemexEngineHandler, ui2::wm::edit_window::EditWindowDelegate};

mod actions {
    use gpui::{App, KeyBinding};

    pub const CONTEXT: &str = "demex-edit-cue-trigger-window";

    gpui::actions!([QuitEditCueTrigger]);

    pub fn init(cx: &mut App) {
        cx.bind_keys([KeyBinding::new("escape", QuitEditCueTrigger, Some(CONTEXT))]);
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

pub struct EditCueTriggerWindow {
    sequence_id: u32,
    cue_idx: CueIdx,

    select_state: Entity<SelectState<Vec<CueTriggerItem>>>,
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
        let select_state = cx
            .new(|cx| SelectState::new(CueTrigger::iter().map_into().collect(), None, window, cx));

        let input_state = cx.new(|cx| InputState::new(window, cx));

        DemexEngineHandler::send_in(
            window,
            cx,
            SequenceCueRequest {
                sequence_id,
                cue_idx,
            },
            {
                let select_state = select_state.clone();
                move |cue, window, cx| {
                    if let Some(cue) = cue {
                        select_state.update(cx, |state, cx| {
                            state.set_selected_value(&cue.trigger, window, cx);
                        });
                    }
                }
            },
        );

        // TODO: subscribe to select state change, and update input state if needed
        let _subscriptions = vec![];

        Self {
            sequence_id,
            cue_idx,

            select_state,
            input_state,

            _subscriptions,
        }
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
            .justify_center()
            .p_4()
            .gap_2()
            .child(Select::new(&self.select_state).w_full())
            .when(
                self.select_state
                    .read(cx)
                    .selected_value()
                    .is_some_and(|val| matches!(val, CueTrigger::Time(_))),
                |this| this.child(NumberInput::new(&self.input_state).suffix("s").w_full()),
            )
    }
}
