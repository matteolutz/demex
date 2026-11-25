use demex_core::command::parser::nodes::action::Action;
use demex_dmx::{DemexDmxOutputConfig, DemexDmxOutputConfigData, debug::DebugOutputVerbosity};
use demex_headless::id::DemexProtoDeviceId;
use gpui::{
    AppContext, Context, Entity, ParentElement, Render, Styled, Window, WindowBounds, div, size,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    form::{field, v_form},
    select::{Select, SelectEvent, SelectItem, SelectState},
};
use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::wm::edit_window::EditWindowDelegate,
};

#[derive(Clone)]
struct UiDebugOutputVerbosity(pub DebugOutputVerbosity);
impl From<DebugOutputVerbosity> for UiDebugOutputVerbosity {
    fn from(value: DebugOutputVerbosity) -> Self {
        Self(value)
    }
}

impl SelectItem for UiDebugOutputVerbosity {
    type Value = Self;

    fn title(&self) -> gpui::SharedString {
        format!("{:?}", self.0).into()
    }

    fn value(&self) -> &Self::Value {
        self
    }
}

pub struct AddDebugOutputWindow {
    selected_verbosity: Entity<SelectState<Vec<UiDebugOutputVerbosity>>>,

    _subscriptions: Vec<gpui::Subscription>,
}

impl AddDebugOutputWindow {
    pub fn new(window: &mut Window, cx: &mut gpui::Context<Self>) -> Self {
        let selected_verbosity = cx.new(|cx| {
            SelectState::new(
                DebugOutputVerbosity::iter().map_into().collect(),
                None,
                window,
                cx,
            )
        });

        let _subscriptions = vec![cx.subscribe(
            &selected_verbosity,
            |this, _, _: &SelectEvent<Vec<UiDebugOutputVerbosity>>, cx| {
                this.set_edited(true, cx);
            },
        )];

        Self {
            selected_verbosity,
            _subscriptions,
        }
    }

    pub fn submit(&self, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(verbosity) = self.selected_verbosity.read(cx).selected_value() else {
            return;
        };

        let mut output_configs = DemexUiState::patch(cx).read(cx).output_configs().to_vec();
        output_configs.push(DemexDmxOutputConfig::new(
            DemexDmxOutputConfigData::Debug(verbosity.0),
            DemexProtoDeviceId::Controller,
        ));

        DemexEngineHandler::engine(cx).exec_ui(Action::UpdateOutputConfigs(output_configs.clone()));

        self.discard_and_close(cx);
    }
}

impl Render for AddDebugOutputWindow {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div().size_full().p_4().child(
            v_form()
                .child(
                    field()
                        .required(true)
                        .label("Debug verbosity")
                        .child(Select::new(&self.selected_verbosity)),
                )
                .child(
                    field().label_indent(false).child(
                        Button::new("submit")
                            .primary()
                            .child("Add")
                            .on_click(cx.listener(|this, _, window, cx| this.submit(window, cx))),
                    ),
                ),
        )
    }
}

impl EditWindowDelegate for AddDebugOutputWindow {
    fn window_title(
        &self,
        _window: &mut gpui::Window,
        _cx: &gpui::App,
    ) -> impl Into<gpui::SharedString> {
        "Add Debug Output"
    }

    fn should_have_save_button(_cx: &gpui::App) -> bool
    where
        Self: Sized,
    {
        false
    }

    fn window_bounds(cx: &mut gpui::App) -> Option<gpui::WindowBounds> {
        Some(WindowBounds::centered(size(500.0.into(), 300.0.into()), cx))
    }

    fn handle_save(&self, _window: &mut gpui::Window, _cx: &mut gpui::App) {}

    fn handle_discard(&self, _window: &mut gpui::Window, _cx: &mut gpui::App) {}
}
