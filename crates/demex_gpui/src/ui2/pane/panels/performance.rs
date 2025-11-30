use std::{collections::HashMap, time::Duration};

use demex_core::{engine::comm::ThreadStatsRequest, utils::thread::DemexThreadStats};
use gpui::{
    App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, Styled, Task, Timer, Window,
};
use gpui_component::{
    dock::{Panel, PanelEvent},
    v_flex,
};

use crate::{engine::DemexEngineHandler, ui2::utils::Deferred};

pub struct PerformancePanel {
    focus_handle: FocusHandle,
    thread_stats: Entity<Deferred<HashMap<String, DemexThreadStats>>>,

    _tasks: Vec<Task<()>>,
}

impl EventEmitter<PanelEvent> for PerformancePanel {}
impl Focusable for PerformancePanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for PerformancePanel {
    fn panel_name(&self) -> &'static str {
        "performance"
    }

    fn title(&self, _window: &Window, _cx: &App) -> gpui::AnyElement {
        "Performance".into_any_element()
    }
}

impl PerformancePanel {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let thread_stats = cx.new(|_| Default::default());

        let tasks = vec![cx.spawn(async move |this, cx| {
            loop {
                let Some(this) = this.upgrade() else {
                    // entity has been dropped
                    break;
                };

                let _ = this.update(cx, |this, cx| {
                    this.update_stats(cx);
                    cx.notify();
                });

                Timer::after(Duration::from_secs(1)).await;
            }
        })];

        Self {
            focus_handle: cx.focus_handle(),
            thread_stats,
            _tasks: tasks,
        }
    }

    fn update_stats(&self, cx: &mut App) {
        self.thread_stats.update(cx, |stats, _| stats.set_loading());
        DemexEngineHandler::send_with(
            cx,
            self.thread_stats.clone(),
            ThreadStatsRequest {},
            |res, this, cx| {
                this.update(res);
                cx.notify();
            },
        );
    }
}

impl Render for PerformancePanel {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        let stats = self.thread_stats.read(cx);
        let element = if let Some(stats) = stats.when_not_loading() {
            format!("{:?}", stats).into_any_element()
        } else {
            "Loading...".into_any_element()
        };

        v_flex()
            .items_center()
            .justify_center()
            .child("Performance")
            .child(element)
    }
}
