use gpui::{
    App, AppContext, Context, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement,
    Render, Styled, Subscription, div,
};
use gpui_component::dock::{Panel, PanelEvent, PanelInfo, register_panel};

use crate::ui2::panels::pool::pool_type::PoolType;

pub mod pool_type;

const POOL_PANEL_NAME: &str = "demex-pool";

pub(super) fn register(cx: &mut App) {
    register_panel(cx, POOL_PANEL_NAME, |_, _, panel_info, _, cx| {
        let pool_type = if let PanelInfo::Panel(panel) = panel_info {
            serde_json::from_value(panel.clone()).ok()
        } else {
            None
        };

        Box::new(cx.new(|cx| PoolPanel::new(pool_type.unwrap_or_else(|| PoolType::default()), cx)))
    });
}

pub struct PoolPanel {
    pool_type: PoolType,

    focus_handle: FocusHandle,

    _subscriptions: Vec<Subscription>,
}

impl PoolPanel {
    pub fn new(pool_type: PoolType, cx: &mut Context<Self>) -> Self {
        let _subscriptions = vec![];

        Self {
            pool_type,
            focus_handle: cx.focus_handle(),
            _subscriptions,
        }
    }
}

impl EventEmitter<PanelEvent> for PoolPanel {}
impl Focusable for PoolPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for PoolPanel {
    fn panel_name(&self) -> &'static str {
        POOL_PANEL_NAME
    }

    fn title(&self, _window: &gpui::Window, _cx: &App) -> gpui::AnyElement {
        format!("{} Pool", self.pool_type).into_any_element()
    }
}

impl Render for PoolPanel {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        div()
            .w_full()
            .h_full()
            .flex()
            .justify_center()
            .items_center()
            .child("Pool")
    }
}
