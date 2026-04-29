use std::{collections::HashMap, time};

use demex_core::command::parser::nodes::action::{
    Action, functions::speedmaster_functions::SpeedMasterTapArgs,
};
use gpui::{
    AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, ParentElement, Render,
    Styled, Subscription, Window,
};
use gpui_component::{
    button::Button,
    dock::PanelEvent,
    h_flex,
    input::{InputEvent, InputState, NumberInput},
    v_flex,
};
use itertools::Itertools;

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::panels::DemexPanel,
};

#[derive(Clone)]
struct SpeedmasterInputState {
    input_state: Entity<InputState>,
    last_tap: Option<time::Instant>,
}

pub struct SpeedmasterPanel {
    focus_handle: FocusHandle,

    input_states: HashMap<u32, SpeedmasterInputState>,

    _subscriptions: Vec<Subscription>,
}

impl SpeedmasterPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_states = DemexUiState::speed_masters(cx)
            .read(cx)
            .clone()
            .into_iter()
            .map(|(id, value)| {
                let input_state =
                    cx.new(|cx| InputState::new(window, cx).default_value(value.bpm.to_string()));

                (
                    id,
                    SpeedmasterInputState {
                        input_state,
                        last_tap: value.last_tap,
                    },
                )
            })
            .collect::<HashMap<_, _>>();

        let mut _subscriptions = vec![cx.observe_in(
            &DemexUiState::speed_masters(cx),
            window,
            |this, speed_masters, window, cx| {
                for (id, new_value) in speed_masters.read(cx).clone().into_iter() {
                    if let Some(input_state) = this.input_states.get_mut(&id) {
                        input_state.input_state.update(cx, |state, cx| {
                            state.set_value(new_value.bpm.to_string(), window, cx)
                        });
                        input_state.last_tap = new_value.last_tap;
                    }

                    cx.notify();
                }
            },
        )];

        _subscriptions.extend(input_states.clone().into_iter().map(|(id, state)| {
            cx.subscribe(
                &state.input_state,
                move |_, state, evt: &InputEvent, cx| match evt {
                    InputEvent::PressEnter { .. } => {
                        let Ok(bpm_value) = state.read(cx).value().parse::<f32>() else {
                            return;
                        };

                        DemexEngineHandler::engine(cx)
                            .exec_ui(Action::SpeedMasterSetBpm(id, bpm_value));
                    }
                    _ => {}
                },
            )
        }));

        Self {
            focus_handle: cx.focus_handle(),
            input_states,
            _subscriptions,
        }
    }
}

impl EventEmitter<PanelEvent> for SpeedmasterPanel {}
impl Focusable for SpeedmasterPanel {
    fn focus_handle(&self, _cx: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl DemexPanel for SpeedmasterPanel {
    fn panel_type() -> super::DockWindowPanelType {
        super::DockWindowPanelType::Speedmaster
    }

    fn deserialize(
        _dock_area: gpui::WeakEntity<gpui_component::dock::DockArea>,
        _panel_state: &gpui_component::dock::PanelState,
        _panel_info: &gpui_component::dock::PanelInfo,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> Self {
        Self::new(window, cx)
    }
}

impl Render for SpeedmasterPanel {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        v_flex().size_full().p_4().gap_2().children(
            self.input_states
                .iter()
                .sorted_by_key(|(id, _)| *id)
                .map(|(id, state)| {
                    h_flex()
                        .gap_2()
                        .child(id.to_string())
                        .child(NumberInput::new(&state.input_state))
                        .child(Button::new(("tap", *id as usize)).label("Tap").on_click({
                            let speedmaster_id = *id;
                            move |_, _, cx| {
                                DemexEngineHandler::engine(cx).exec_ui(Action::SpeedMasterTap(
                                    SpeedMasterTapArgs { speedmaster_id },
                                ));
                            }
                        }))
                }),
        )
    }
}
