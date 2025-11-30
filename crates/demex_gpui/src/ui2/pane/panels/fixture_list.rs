use gpui::{
    AppContext, Entity, FocusHandle, IntoElement, ParentElement, Render, SharedString, Styled, div,
};
use gpui_component::dock::PanelView;

struct FixtureListPanelRender {
    content: SharedString,
}

impl FixtureListPanelRender {
    fn new(content: impl Into<SharedString>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl Render for FixtureListPanelRender {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        div()
            .w_full()
            .h_full()
            .flex()
            .justify_center()
            .items_center()
            .child(self.content.clone())
            .into_any_element()
    }
}

pub struct FixtureListPanel {
    focus_handle: FocusHandle,
    view: Entity<FixtureListPanelRender>,
}

impl FixtureListPanel {
    pub fn new(content: impl Into<SharedString>, cx: &mut gpui::App) -> Self {
        Self {
            view: cx.new(|_| FixtureListPanelRender::new(content)),
            focus_handle: cx.focus_handle(),
        }
    }
}

impl PanelView for FixtureListPanel {
    fn panel_name(&self, cx: &gpui::App) -> &'static str {
        "Fixture List"
    }

    fn panel_id(&self, cx: &gpui::App) -> gpui::EntityId {
        self.view.entity_id()
    }

    fn tab_name(&self, cx: &gpui::App) -> Option<gpui::SharedString> {
        None
    }

    fn title(&self, window: &gpui::Window, cx: &gpui::App) -> gpui::AnyElement {
        "Fixture List".into_any_element()
    }

    fn title_suffix(
        &self,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> Option<gpui::AnyElement> {
        None
    }

    fn title_style(&self, cx: &gpui::App) -> Option<gpui_component::dock::TitleStyle> {
        None
    }

    fn closable(&self, cx: &gpui::App) -> bool {
        false
    }

    fn zoomable(&self, cx: &gpui::App) -> Option<gpui_component::dock::PanelControl> {
        None
    }

    fn visible(&self, cx: &gpui::App) -> bool {
        true
    }

    fn set_active(&self, active: bool, window: &mut gpui::Window, cx: &mut gpui::App) {
        // TODO
    }

    fn set_zoomed(&self, zoomed: bool, window: &mut gpui::Window, cx: &mut gpui::App) {
        // TODO
    }

    fn on_added_to(
        &self,
        tab_panel: gpui::WeakEntity<gpui_component::dock::TabPanel>,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) {
        // TODO
    }

    fn on_removed(&self, window: &mut gpui::Window, cx: &mut gpui::App) {
        // TODO
    }

    fn dropdown_menu(
        &self,
        menu: gpui_component::menu::PopupMenu,
        window: &gpui::Window,
        cx: &gpui::App,
    ) -> gpui_component::menu::PopupMenu {
        menu
    }

    fn toolbar_buttons(
        &self,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> Option<Vec<gpui_component::button::Button>> {
        None
    }

    fn view(&self) -> gpui::AnyView {
        self.view.clone().into()
    }

    fn focus_handle(&self, cx: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }

    fn dump(&self, cx: &gpui::App) -> gpui_component::dock::PanelState {
        todo!()
    }

    fn inner_padding(&self, cx: &gpui::App) -> bool {
        false
    }
}
