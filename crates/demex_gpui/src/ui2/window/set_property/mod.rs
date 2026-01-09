use demex_core::{command::parser::nodes::object::Object, engine::comm::ObjectPropertyRequest};
use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription,
    Window, WindowBounds, div, prelude::FluentBuilder, size,
};

mod property_type;
use gpui_component::{
    input::{Input, InputState, NumberInput},
    v_flex,
};
pub use property_type::*;

use crate::{engine::DemexEngineHandler, ui2::wm::edit_window::EditWindowDelegate};

pub(super) fn init(_cx: &mut App) {}

pub struct SetPropertyWindow {
    object: Object,
    property: String,
    property_type: SetPropertyWindowPropertyType,

    value: Entity<Option<String>>,
    input_state: Entity<InputState>,

    _subscriptions: Vec<Subscription>,
}

impl SetPropertyWindow {
    pub fn new(
        object: Object,
        property: impl ToString,
        property_type: SetPropertyWindowPropertyType,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let property = property.to_string();
        let value = cx.new(|_| None);
        let input_state = cx.new(|cx| InputState::new(window, cx));

        DemexEngineHandler::send_with(
            cx,
            value.clone(),
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
                    });
                }
                cx.notify();
            }),
            cx.observe(&input_state, |this, state, cx| {
                let value = state.read(cx).value();
                match this.property_type {
                    _ => {}
                }
            }),
        ];

        Self {
            object,
            property,
            property_type,
            value,
            input_state,
            _subscriptions,
        }
    }
}

impl SetPropertyWindow {
    pub fn render_input(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        match self.property_type {
            SetPropertyWindowPropertyType::String => {
                Input::new(&self.input_state).w_full().into_any_element()
            }
            SetPropertyWindowPropertyType::Integer { range: _ } => {
                NumberInput::new(&self.input_state)
                    .placeholder(self.property.clone())
                    .into_any_element()
            }
            SetPropertyWindowPropertyType::RelativeTime { unit } => {
                NumberInput::new(&self.input_state)
                    .placeholder(self.property.clone())
                    .suffix(unit.get_suffix().to_string())
                    .into_any_element()
            }
            _ => v_flex().child("TODO").into_any_element(),
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
            .size_full()
            .p_4()
            .gap_4()
            .child(self.render_input(window, cx))
            .child(div().w_full().flex_1())
    }
}

impl EditWindowDelegate for SetPropertyWindow {
    fn window_title(&self, _window: &mut gpui::Window, _cx: &App) -> impl Into<gpui::SharedString> {
        format!("Set {:?} for {}", self.property, self.object)
    }

    fn window_bounds(cx: &mut gpui::App) -> Option<gpui::WindowBounds> {
        Some(WindowBounds::centered(size(500.0.into(), 300.0.into()), cx))
    }

    fn handle_save(&self, _window: &mut gpui::Window, _cx: &mut App) {
        // TODO
    }

    fn handle_discard(&self, _window: &mut gpui::Window, _cx: &mut App) {
        // TODO
    }
}
