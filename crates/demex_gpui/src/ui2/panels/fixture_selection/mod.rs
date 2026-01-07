use demex_core::{
    command::parser::nodes::{
        action::{Action, functions::set_function::ObjectSetPropertyArgs},
        object::{HomeableObject, Object},
    },
    event::FixtureSelectionWithGroup,
    selection::FixtureSelectionProperty,
};
use gpui::{
    App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, SharedString, Styled, Subscription, Window, div,
};
use gpui_component::{
    button::Button,
    checkbox::Checkbox,
    dock::{Panel, PanelEvent, register_panel},
};
use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::{
        ext::GpuiContextExtension,
        panels::{
            fixture_selection::property::{
                FixtureSelectionPropertyExt, FixtureSelectionPropertyType,
            },
            toolbar_buttons,
        },
    },
};

mod property;

const FIXTURE_SELECTION_PANEL_NAME: &str = "demex-fixture-selection";

pub(super) fn register(cx: &mut App) {
    register_panel(cx, FIXTURE_SELECTION_PANEL_NAME, |_, _, _, _, cx| {
        Box::new(cx.new(|cx| FixtureSelectionPanel::new(cx)))
    });
}

pub struct FixtureSelectionPanel {
    focus_handle: FocusHandle,

    fixture_selection: Entity<Option<FixtureSelectionWithGroup>>,

    _subscriptions: Vec<Subscription>,
}

impl FixtureSelectionPanel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let fixture_selection = DemexUiState::fixture_selection(cx);

        let _subscriptions = vec![cx.observe_and_notify(&fixture_selection)];

        Self {
            focus_handle: cx.focus_handle(),
            fixture_selection,
            _subscriptions,
        }
    }
}

impl EventEmitter<PanelEvent> for FixtureSelectionPanel {}
impl Focusable for FixtureSelectionPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for FixtureSelectionPanel {
    fn panel_name(&self) -> &'static str {
        FIXTURE_SELECTION_PANEL_NAME
    }

    fn title(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        "Fixture Selection"
    }

    fn toolbar_buttons(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Vec<Button>> {
        Some(toolbar_buttons(self, window, cx))
    }
}

impl FixtureSelectionPanel {
    fn render_integer_property(
        &mut self,
        property: FixtureSelectionProperty,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let Some(selection) = self.fixture_selection.read(cx) else {
            return div();
        };

        let value = property.get_as_usize(selection.selection());

        let set_value = |property_name: String, value: usize, cx: &mut App| {
            DemexEngineHandler::engine(cx).exec_ui(Action::ObjectSetProperty(
                ObjectSetPropertyArgs {
                    object: Object::HomeableObject(HomeableObject::CurrentFixtureSelection),
                    key: property_name,
                    value: value.to_string(),
                },
            ));
        };

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .child(property.to_string())
            .child(
                Button::new(SharedString::from(format!("dec-{}", property.to_string())))
                    .label("-")
                    .on_click(move |_, _, cx| set_value(property.to_string(), value - 1, cx)),
            )
            .child(value.to_string())
            .child(
                Button::new(SharedString::from(format!("inc-{}", property.to_string())))
                    .label("+")
                    .on_click(move |_, _, cx| set_value(property.to_string(), value + 1, cx)),
            )
    }

    fn render_bool_property(
        &mut self,
        property: FixtureSelectionProperty,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let Some(selection) = self.fixture_selection.read(cx) else {
            return div();
        };

        let value = property.get_as_bool(selection.selection());

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .child(
                Checkbox::new(SharedString::from(format!("flag-{}", property.to_string())))
                    .checked(value)
                    .on_click(move |checked, _, cx| {
                        DemexEngineHandler::engine(cx).exec_ui(Action::ObjectSetProperty(
                            ObjectSetPropertyArgs {
                                object: Object::HomeableObject(
                                    HomeableObject::CurrentFixtureSelection,
                                ),
                                key: property.to_string(),
                                value: checked.to_string(),
                            },
                        ));
                    }),
            )
            .child(property.to_string())
    }
}

impl Render for FixtureSelectionPanel {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .child(format!(
                "Total offsets: {}",
                self.fixture_selection
                    .read(cx)
                    .as_ref()
                    .map(|sel| sel.selection().num_offsets().to_string())
                    .unwrap_or_else(|| "-".to_string())
            ))
            .child(format!(
                "Fixtures: {}",
                self.fixture_selection
                    .read(cx)
                    .as_ref()
                    .map(|sel| sel
                        .selection()
                        .fixtures()
                        .iter()
                        .map(|f| format!("{}", f))
                        .join(", "))
                    .unwrap_or_else(|| "-".to_string())
            ))
            .children(FixtureSelectionProperty::iter().map(|property| {
                match property.get_type() {
                    FixtureSelectionPropertyType::Flag => self
                        .render_bool_property(property, window, cx)
                        .into_any_element(),
                    FixtureSelectionPropertyType::Integer => self
                        .render_integer_property(property, window, cx)
                        .into_any_element(),
                }
            }))
            .into_any_element()
    }
}
