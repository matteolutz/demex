use demex_core::updatables::runtime::RuntimePhase;
use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription, Window,
    prelude::FluentBuilder,
};
use gpui_component::{
    IndexPath, h_flex,
    input::{InputEvent, InputState, NumberInput},
    select::{Select, SelectEvent, SelectItem, SelectState},
    v_flex,
};
use itertools::Itertools;
use strum::IntoEnumIterator;

#[derive(Debug, Copy, Clone)]
struct RuntimePhaseItem(RuntimePhase);

impl From<RuntimePhase> for RuntimePhaseItem {
    fn from(phase: RuntimePhase) -> Self {
        RuntimePhaseItem(phase)
    }
}

impl SelectItem for RuntimePhaseItem {
    type Value = RuntimePhase;

    fn title(&self) -> gpui::SharedString {
        match &self.0 {
            RuntimePhase::Single(_) => "Single".into(),
            RuntimePhase::Range { .. } => "Range".into(),
        }
    }

    fn value(&self) -> &Self::Value {
        &self.0
    }
}

pub struct RuntimePhaseEditor {
    runtime_phase: RuntimePhase,

    select_state: Entity<SelectState<Vec<RuntimePhaseItem>>>,
    start_input_state: Entity<InputState>,
    end_input_state: Entity<InputState>,

    _subscriptions: Vec<Subscription>,
}

impl RuntimePhaseEditor {
    pub fn new(runtime_phase: RuntimePhase, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let select_state = cx.new(|cx| {
            SelectState::new(RuntimePhase::iter().map_into().collect(), None, window, cx)
        });

        let start_input_state = cx.new(|cx| InputState::new(window, cx));
        let end_input_state = cx.new(|cx| InputState::new(window, cx));

        let _subscriptions = vec![
            cx.subscribe_in(
                &select_state,
                window,
                |this, _, evt: &SelectEvent<Vec<RuntimePhaseItem>>, window, cx| match evt {
                    SelectEvent::Confirm(runtime_phase) => {
                        if let Some(runtime_phase) = runtime_phase {
                            this.runtime_phase = *runtime_phase;
                            this.update_inputs(*runtime_phase, window, cx);

                            cx.notify();
                        }
                    }
                },
            ),
            cx.subscribe(
                &start_input_state,
                |this, input, evt: &InputEvent, cx| match evt {
                    InputEvent::Change => match &mut this.runtime_phase {
                        RuntimePhase::Single(value) | RuntimePhase::Range { start: value, .. } => {
                            if let Ok(input_value) = input.read(cx).value().parse() {
                                *value = input_value;
                                cx.notify();
                            }
                        }
                    },
                    _ => {}
                },
            ),
            cx.subscribe(
                &end_input_state,
                |this, input, evt: &InputEvent, cx| match evt {
                    InputEvent::Change => match &mut this.runtime_phase {
                        RuntimePhase::Range { end: value, .. } => {
                            if let Ok(input_value) = input.read(cx).value().parse() {
                                *value = input_value;
                                cx.notify();
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                },
            ),
        ];

        let s = Self {
            runtime_phase,

            select_state,
            start_input_state,
            end_input_state,

            _subscriptions,
        };

        s.update_inputs(runtime_phase, window, cx);

        s
    }

    fn update_inputs(
        &self,
        runtime_phase: RuntimePhase,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match runtime_phase {
            RuntimePhase::Single(value) => self.start_input_state.update(cx, |input, cx| {
                input.set_value(value.to_string(), window, cx)
            }),
            RuntimePhase::Range { start, end } => {
                self.start_input_state.update(cx, |input, cx| {
                    input.set_value(start.to_string(), window, cx)
                });
                self.end_input_state
                    .update(cx, |input, cx| input.set_value(end.to_string(), window, cx));
            }
        }
    }

    pub fn runtime_phase(&self) -> RuntimePhase {
        self.runtime_phase
    }

    pub fn set_runtime_phase(
        &mut self,
        runtime_phase: impl Into<RuntimePhase>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let runtime_phase = runtime_phase.into();

        self.runtime_phase = runtime_phase;
        self.select_state.update(cx, |state, cx| {
            state.set_selected_index(
                RuntimePhase::iter()
                    .find_position(|phase| {
                        std::mem::discriminant(phase) == std::mem::discriminant(&runtime_phase)
                    })
                    .map(|(idx, _)| IndexPath::new(idx)),
                window,
                cx,
            )
        });
        self.update_inputs(self.runtime_phase, window, cx);
        cx.notify();
    }
}

impl Render for RuntimePhaseEditor {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .gap_2()
            .child(Select::new(&self.select_state))
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        NumberInput::new(&self.start_input_state)
                            .suffix("°")
                            .w_full()
                            .min_w_40(),
                    )
                    .when(
                        matches!(self.runtime_phase, RuntimePhase::Range { .. }),
                        |this| {
                            this.child("thru").child(
                                NumberInput::new(&self.end_input_state)
                                    .suffix("°")
                                    .w_full()
                                    .min_w_40(),
                            )
                        },
                    ),
            )
    }
}
