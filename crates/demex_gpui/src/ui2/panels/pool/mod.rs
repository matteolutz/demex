use gpui::{
    App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, Styled, Subscription, Window, div,
};
use gpui_component::{
    button::Button,
    dock::{Panel, PanelEvent, PanelInfo, register_panel},
};

use crate::ui2::panels::{
    pool::{pool::Pool, pool_type::PoolType},
    toolbar_buttons,
};

mod pool;
mod pool_button;
mod pool_quick_actions;
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
    focus_handle: FocusHandle,

    pool_type: PoolType,
    pool: Entity<Pool>,

    _subscriptions: Vec<Subscription>,
}

impl PoolPanel {
    pub fn new(pool_type: PoolType, cx: &mut Context<Self>) -> Self {
        let pool = cx.new(|cx| Pool::new(cx));

        let _subscriptions = vec![];

        Self {
            focus_handle: cx.focus_handle(),
            pool_type,
            pool,
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

    fn title(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        format!("{} Pool", self.pool_type)
    }

    fn toolbar_buttons(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Vec<Button>> {
        Some(toolbar_buttons(self, window, cx))
    }
}

impl Render for PoolPanel {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        div().size_full().child(self.pool.clone())
    }
}
