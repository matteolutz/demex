use std::sync::Arc;

use demex_core::patch::Patch;
use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement, Render,
    Styled, Subscription, div,
};
use gpui_component::dock::{Panel, PanelEvent};
use itertools::Itertools;

use crate::{engine::state::DemexUiState, ui2::ext::GpuiContextExtension};

pub struct FixtureListPanel {
    focus_handle: FocusHandle,

    patch: Entity<Arc<Patch>>,

    _subscriptions: Vec<Subscription>,
}

impl FixtureListPanel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let patch = DemexUiState::patch(cx);

        let _subscriptions = vec![cx.observe_and_notify(&patch)];

        Self {
            focus_handle: cx.focus_handle(),
            patch,
            _subscriptions,
        }
    }
}

impl EventEmitter<PanelEvent> for FixtureListPanel {}
impl Focusable for FixtureListPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for FixtureListPanel {
    fn panel_name(&self) -> &'static str {
        "fixture-list"
    }

    fn title(&self, _window: &gpui::Window, _cx: &App) -> gpui::AnyElement {
        "Fixture List".into_any_element()
    }
}

impl Render for FixtureListPanel {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        let patch = self.patch.read(cx);

        div()
            .w_full()
            .h_full()
            .flex()
            .justify_center()
            .items_center()
            .child(patch.fixtures().map(|f| f.name()).join(", "))
            .into_any_element()
    }
}
