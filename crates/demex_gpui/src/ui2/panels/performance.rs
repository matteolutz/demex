use gpui::{
    App, Context, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement, Render, Styled,
    Subscription, Window, div, prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, StyledExt, chart::AreaChart, dock::PanelEvent, scroll::ScrollableElement, v_flex,
};

use crate::{
    engine::state::DemexUiState,
    ui2::{ext::GpuiContextExtension, panels::DemexPanel},
};

pub struct PerformancePanel {
    focus_handle: FocusHandle,

    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<PanelEvent> for PerformancePanel {}
impl Focusable for PerformancePanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl DemexPanel for PerformancePanel {
    fn panel_type() -> super::DockWindowPanelType {
        super::DockWindowPanelType::PerformancePanel
    }

    fn deserialize(
        _dock_area: gpui::WeakEntity<gpui_component::dock::DockArea>,
        _panel_state: &gpui_component::dock::PanelState,
        _panel_info: &gpui_component::dock::PanelInfo,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        PerformancePanel::new(window, cx)
    }
}

impl PerformancePanel {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let performance_data = DemexUiState::performance(cx);

        let _subscriptions = vec![cx.observe_and_notify(&performance_data)];

        Self {
            focus_handle: cx.focus_handle(),
            _subscriptions,
        }
    }
}

impl Render for PerformancePanel {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        let performance = DemexUiState::performance(cx).read(cx).clone();

        div().w_full().overflow_scrollbar().child(
            v_flex()
                .w_full()
                .min_w_64()
                .p_4()
                .gap_4()
                .justify_center()
                .child(div().font_bold().text_2xl().child("Performance"))
                .children(performance.into_iter().map(|(thread_name, stats)| {
                    div().w_full().child(stats_chart_container(
                        thread_name,
                        stats
                            .current_its()
                            .map(|its| format!("Current: {:.0}it/s", its)),
                        AreaChart::new(
                            stats
                                .move_data()
                                .enumerate()
                                .map(|(idx, p)| (idx, p.as_ref().map(|p| 1.0 / p.dt()), p)),
                        )
                        .step_after()
                        .x(|(idx, _, _)| idx.to_string())
                        .y(|(_, its, _)| {
                            its.as_ref()
                                .and_then(|its| its.is_finite().then(|| *its))
                                .unwrap_or(0.0)
                        }),
                        false,
                        cx,
                    ))
                })),
        )
    }
}

fn stats_chart_container(
    title: String,
    subtitle: Option<String>,
    chart: impl IntoElement,
    center: bool,
    cx: &App,
) -> impl IntoElement {
    v_flex()
        .h_full()
        .border_1()
        .border_color(cx.theme().border)
        .rounded_lg()
        .p_4()
        .min_w_64()
        .min_h_64()
        .child(
            div()
                .when(center, |this| this.text_center())
                .font_semibold()
                .child(title),
        )
        .when(subtitle.is_some(), |this| {
            this.child(
                div()
                    .when(center, |this| this.text_center())
                    .text_sm()
                    .child(subtitle.unwrap()),
            )
        })
        .child(
            div()
                .when(center, |this| this.text_center())
                .text_color(cx.theme().muted_foreground)
                .text_sm()
                .child("Data period label"),
        )
        .child(div().flex_1().pt_4().child(chart))
}
