use std::time::Duration;

use gpui::{
    Context, EventEmitter, FocusHandle, Focusable, ParentElement, Render, Styled, Task, Window,
};
use gpui_component::{ActiveTheme, dock::PanelEvent, h_flex};

use crate::ui2::panels::DemexPanel;

pub struct ClockPanel {
    focus_handle: FocusHandle,

    _tick_task: Option<Task<()>>,
}

impl ClockPanel {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let tick_task = cx.spawn(async |this, cx| {
            loop {
                let _ = this.update(cx, |_, cx| cx.notify());
                cx.background_executor()
                    .timer(Duration::from_secs_f64(1.0 / 30.0))
                    .await;
            }
        });

        Self {
            focus_handle: cx.focus_handle(),
            _tick_task: Some(tick_task),
        }
    }
}

impl EventEmitter<PanelEvent> for ClockPanel {}
impl Focusable for ClockPanel {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl DemexPanel for ClockPanel {
    fn panel_type() -> super::DockWindowPanelType {
        super::DockWindowPanelType::Clock
    }

    fn deserialize(
        _dock_area: gpui::WeakEntity<gpui_component::dock::DockArea>,
        _panel_state: &gpui_component::dock::PanelState,
        _panel_info: &gpui_component::dock::PanelInfo,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::new(window, cx)
    }
}

impl Render for ClockPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let local_now = chrono::Local::now();

        h_flex()
            .size_full()
            .justify_center()
            .items_center()
            .text_3xl()
            .font_family("JetBrains Mono")
            .text_color(cx.theme().green)
            .child(local_now.format("%H:%M:%S").to_string())
    }
}
