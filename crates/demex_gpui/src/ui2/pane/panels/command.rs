use gpui::{App, AppContext, Entity, FocusHandle, IntoElement, ParentElement, Render, Styled, div};
use gpui_component::dock::PanelView;

struct CommandPanelRender {}

impl CommandPanelRender {
    fn new(_cx: &mut App) -> Self {
        Self {}
    }
}

impl Render for CommandPanelRender {
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
            .child("Command")
            .into_any_element()
    }
}

pub struct CommandPanel {
    focus_handle: FocusHandle,
    view: Entity<CommandPanelRender>,
}

impl CommandPanel {
    pub fn new(cx: &mut gpui::App) -> Self {
        Self {
            view: cx.new(|cx| CommandPanelRender::new(cx)),
            focus_handle: cx.focus_handle(),
        }
    }
}

impl PanelView for CommandPanel {
    fn panel_name(&self, cx: &gpui::App) -> &'static str {
        "Command"
    }

    fn panel_id(&self, cx: &gpui::App) -> gpui::EntityId {
        self.view.entity_id()
    }

    fn tab_name(&self, cx: &gpui::App) -> Option<gpui::SharedString> {
        None
    }

    fn title(&self, window: &gpui::Window, cx: &gpui::App) -> gpui::AnyElement {
        "Command".into_any_element()
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
