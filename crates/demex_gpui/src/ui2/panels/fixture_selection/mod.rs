use demex_core::{
    command::parser::nodes::{
        action::{Action, functions::set_function::ObjectSetPropertyArgs},
        object::{HomeableObject, Object},
    },
    event::FixtureSelectionWithGroup,
    selection::FixtureSelectionProperty,
};
use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, SharedString, Styled, Subscription, Window, div, prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, button::Button, checkbox::Checkbox, dock::PanelEvent, h_flex,
    scroll::ScrollableElement, v_flex,
};
use strum::IntoEnumIterator;

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::{
        ext::GpuiContextExtension,
        panels::{
            DemexPanel,
            fixture_selection::property::{
                FixtureSelectionPropertyExt, FixtureSelectionPropertyType,
            },
        },
    },
};

mod property;

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

impl DemexPanel for FixtureSelectionPanel {
    fn panel_type() -> super::DockWindowPanelType {
        super::DockWindowPanelType::FixtureSelection
    }

    fn deserialize(
        _dock_area: gpui::WeakEntity<gpui_component::dock::DockArea>,
        _panel_state: &gpui_component::dock::PanelState,
        _panel_info: &gpui_component::dock::PanelInfo,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        FixtureSelectionPanel::new(cx)
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
            .child(div().text_right().w_20().child(property.to_string()))
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
            .when_some(
                self.fixture_selection.read(cx).as_ref(),
                |this, selection| {
                    this.child(
                        div().w_full().h_auto().max_h_64().p_4().child(
                            div()
                                .size_full()
                                .id("fixture-selection-offset-list-outer")
                                .overflow_x_scrollbar()
                                .py_2()
                                .border_1()
                                .border_color(cx.theme().border)
                                .child(
                                    h_flex()
                                        .border_b_1()
                                        .border_color(cx.theme().border)
                                        .px_2()
                                        .gap_2()
                                        .mb_2()
                                        .children((0..selection.selection().num_offsets()).map(
                                            |offset| {
                                                let total_offset =
                                                    selection.selection().num_offsets();
                                                let offset_deg =
                                                    (offset as f32 / total_offset as f32) * 360.0;
                                                h_flex()
                                                    .justify_center()
                                                    .w_20()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .text_sm()
                                                    .child(format!(
                                                        "{} ({:.0}°)",
                                                        offset, offset_deg
                                                    ))
                                            },
                                        )),
                                )
                                .child(
                                    h_flex()
                                        .items_start()
                                        .id("fixture-selection-offset-list-inner")
                                        .size_full()
                                        .overflow_y_scrollbar()
                                        .px_2()
                                        .gap_2()
                                        .children((0..selection.selection().num_offsets()).map(
                                            |offset| {
                                                // 5 rem per fixture
                                                v_flex().gap_2().justify_start().children(
                                                    selection
                                                        .selection()
                                                        .fixtures_with_offset_idx(offset)
                                                        .map(|f_path| {
                                                            v_flex()
                                                                .justify_center()
                                                                .items_center()
                                                                .p_1()
                                                                .w_20()
                                                                .h_20()
                                                                .bg(cx.theme().secondary)
                                                                .child(f_path.to_string())
                                                        }),
                                                )
                                            },
                                        )),
                                ),
                        ),
                    )
                },
            )
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
