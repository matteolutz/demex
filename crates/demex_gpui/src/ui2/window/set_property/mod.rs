use demex_core::{
    command::parser::nodes::{
        action::{Action, functions::set_function::ObjectSetPropertyArgs},
        object::Object,
    },
    engine::comm::ObjectPropertyRequest,
};
use gpui::{
    App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    Styled, Subscription, Window, WindowBounds, div, prelude::FluentBuilder, size,
};

mod property_type;
use gpui_component::{
    input::{Input, InputEvent, InputState, NumberInput},
    v_flex,
};
pub use property_type::*;

mod grid;

use crate::{
    engine::DemexEngineHandler,
    ui2::{
        components::number_input_grid::{NumberInputGrid, NumberInputGridEvent},
        window::set_property::grid::NumberInputModeExt,
        wm::edit_window::EditWindowDelegate,
    },
};

mod actions {
    use gpui::{App, KeyBinding};

    pub const CONTEXT: &str = "demex-set-property-window";

    gpui::actions!([CloseSetProperty]);

    pub fn init(cx: &mut App) {
        cx.bind_keys([KeyBinding::new("escape", CloseSetProperty, Some(CONTEXT))]);
    }
}

pub(super) fn init(cx: &mut App) {
    actions::init(cx);
}

pub struct SetPropertyWindow {
    object: Object,
    property: String,
    property_type: SetPropertyWindowPropertyType,

    _value: Entity<Option<String>>,
    input_state: Entity<InputState>,

    _subscriptions: Vec<Subscription>,
}

impl SetPropertyWindow {
    pub fn new(
        object: impl Into<Object>,
        property: impl ToString,
        property_type: SetPropertyWindowPropertyType,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let object = object.into();

        match property_type.number_input_mode() {
            NumberInputMode::Float { .. } | NumberInputMode::Integer { .. } => {
                window.resize(size(350.0.into(), 400.0.into()));
            }
            NumberInputMode::None => {}
        }

        let property = property.to_string();
        let value = cx.new(|_| None);
        let input_state = cx
            .new(|cx| InputState::new(window, cx).validate(property_type.clone().get_validator()));

        DemexEngineHandler::send_with(
            value.clone(),
            cx,
            ObjectPropertyRequest {
                object: object.clone(),
                property: property.clone(),
            },
            move |val, value, cx| {
                if let Some(val) = val {
                    *value = Some(val);
                    cx.notify();
                }
            },
        );

        let _subscriptions = vec![
            cx.observe_in(&value, window, |this, value, window, cx| {
                if let Some(value) = value.read(cx).clone() {
                    this.input_state.update(cx, |state, cx| {
                        state.set_value(value, window, cx);
                        state.focus(window, cx);
                        state.select_all_content(cx);
                    });
                }
                cx.notify();
            }),
            cx.subscribe(&input_state, |this, state, evt: &InputEvent, cx| {
                match evt {
                    InputEvent::Change => {
                        let _value = state.read(cx).value();
                        // TODO: check for validity
                    }
                    InputEvent::PressEnter { .. } => {
                        this.submit(cx);
                    }
                    _ => {}
                }
            }),
        ];

        Self {
            object,
            property,
            property_type,
            _value: value,
            input_state,
            _subscriptions,
        }
    }

    fn submit(&self, cx: &mut App) {
        let value = self.input_state.read(cx).value();

        DemexEngineHandler::engine(cx).exec_ui(Action::ObjectSetProperty(ObjectSetPropertyArgs {
            object: self.object.clone(),
            key: self.property.clone(),
            value: value.into(),
        }));

        self.close(cx);
    }
}

impl SetPropertyWindow {
    pub fn render_number_input_grid(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let Some(buttons) = self
            .property_type
            .number_input_mode()
            .get_number_grid_buttons()
        else {
            return div();
        };

        div().size_full().child(
            NumberInputGrid::new(3, 5)
                .buttons(buttons)
                .on_click(cx.listener(|this, event, window, cx| match event {
                    NumberInputGridEvent::Insert(text) => {
                        this.input_state.update(cx, |state, cx| {
                            state.insert(text, window, cx);
                        });
                    }
                    NumberInputGridEvent::Delete => this.input_state.update(cx, |state, cx| {
                        if state.cursor() == 0 {
                            return;
                        }

                        let mut text = state.value().to_string();
                        text.remove(state.cursor() - 1);
                        state.set_value(text, window, cx);
                    }),
                    NumberInputGridEvent::Submit => {
                        this.submit(cx);
                    }
                })),
        )
    }

    pub fn render_input(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        match self.property_type {
            SetPropertyWindowPropertyType::String => {
                Input::new(&self.input_state).w_full().into_any_element()
            }
            SetPropertyWindowPropertyType::Integer { range: _ }
            | SetPropertyWindowPropertyType::Float { range: _ }
            | SetPropertyWindowPropertyType::Percentage => NumberInput::new(&self.input_state)
                .w_full()
                .placeholder(self.property.clone())
                .when(
                    matches!(
                        self.property_type,
                        SetPropertyWindowPropertyType::Percentage
                    ),
                    |this| this.suffix("%"),
                )
                .into_any_element(),
            SetPropertyWindowPropertyType::RelativeTime {
                unit,
                allow_negative: _,
            } => NumberInput::new(&self.input_state)
                .w_full()
                .placeholder(self.property.clone())
                .suffix(unit.get_suffix().to_string())
                .into_any_element(),
        }
    }
}

impl Render for SetPropertyWindow {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> impl gpui::IntoElement {
        v_flex()
            .key_context(actions::CONTEXT)
            .on_action(cx.listener(|this, _: &actions::CloseSetProperty, _, cx| {
                this.close(cx);
            }))
            .size_full()
            .p_4()
            .gap_4()
            .items_center()
            .justify_center()
            .child(self.render_input(window, cx))
            .child(self.render_number_input_grid(window, cx))
    }
}

impl EditWindowDelegate for SetPropertyWindow {
    fn window_title(&self, _window: &mut gpui::Window, _cx: &App) -> impl Into<gpui::SharedString> {
        format!("Set {:?} for {}", self.property, self.object)
    }

    fn window_bounds(cx: &mut gpui::App) -> Option<gpui::WindowBounds> {
        Some(WindowBounds::centered(size(500.0.into(), 100.0.into()), cx))
    }

    fn handle_save(&self, _window: &mut gpui::Window, _cx: &mut App) {}

    fn handle_discard(&self, _window: &mut gpui::Window, _cx: &mut App) {}
}
