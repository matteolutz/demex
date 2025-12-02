use demex_core::{
    command::parser::nodes::{
        action::{Action, functions::set_function::ObjectSetPropertyArgs},
        object::{HomeableObject, Object},
    },
    selection::FixtureSelection,
};
use gpui::{
    App, AppContext, ClickEvent, Context, Entity, EventEmitter, FocusHandle, Focusable,
    IntoElement, ParentElement, Render, Styled, Subscription, Window, div,
};
use gpui_component::{
    button::Button,
    dock::{Panel, PanelEvent, register_panel},
};

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::ext::GpuiContextExtension,
};

const FIXTURE_SELECTION_PANEL_NAME: &str = "demex-fixture-selection";

pub(super) fn register(cx: &mut App) {
    register_panel(cx, FIXTURE_SELECTION_PANEL_NAME, |_, _, _, _, cx| {
        Box::new(cx.new(|cx| FixtureSelectionPanel::new(cx)))
    });
}

pub struct FixtureSelectionPanel {
    focus_handle: FocusHandle,

    fixture_selection: Entity<Option<FixtureSelection>>,

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

    fn title(&self, _window: &gpui::Window, _cx: &App) -> gpui::AnyElement {
        "Fixture Selection".into_any_element()
    }
}

impl FixtureSelectionPanel {
    fn handle_wing_inc(&mut self, _evt: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(sel) = self.fixture_selection.read(cx) else {
            return;
        };

        DemexEngineHandler::engine(cx).exec_ui(Action::ObjectSetProperty(ObjectSetPropertyArgs {
            object: Object::HomeableObject(HomeableObject::CurrentFixtureSelection),
            key: "Wings".into(),
            value: (sel.wings() + 1).to_string(),
        }));
    }
}

impl Render for FixtureSelectionPanel {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        div()
            .w_full()
            .h_full()
            .flex()
            .justify_center()
            .items_center()
            .child(format!("{:?}", self.fixture_selection.read(cx)))
            .child(
                Button::new("inc-wing")
                    .label("Wing+")
                    .on_click(cx.listener(Self::handle_wing_inc)),
            )
            .into_any_element()
    }
}
