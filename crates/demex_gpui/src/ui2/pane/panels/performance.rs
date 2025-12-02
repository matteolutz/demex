use gpui::{
    App, Context, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement, Render, Styled,
    Subscription, Window, div, prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, StyledExt, WindowExt,
    button::Button,
    chart::AreaChart,
    dock::{Panel, PanelEvent},
    v_flex,
};

use crate::{engine::state::DemexUiState, ui2::ext::GpuiContextExtension};

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

        v_flex()
            .w_full()
            .p_4()
            .gap_4()
            .justify_center()
            .child(div().font_bold().text_2xl().child("Performance"))
            .children(performance.into_iter().map(|(thread_name, stats)| {
                stats_chart_container(
                    thread_name,
                    stats
                        .current_fps()
                        .map(|fps| format!("Current: {:.0}fps", fps)),
                    AreaChart::new(
                        stats
                            .move_data()
                            .enumerate()
                            .map(|(idx, p)| (idx, p.as_ref().map(|p| 1.0 / p.dt()), p)),
                    )
                    .step_after()
                    .x(|(idx, _, _)| idx.to_string())
                    .y(|(_, fps, _)| {
                        fps.as_ref()
                            .and_then(|fps| fps.is_finite().then(|| *fps))
                            .unwrap_or(0.0)
                    }),
                    false,
                    cx,
                )
            }))
            .child(
                Button::new("test")
                    .label("Dialog öffnen")
                    .on_click(cx.listener(|_, _, window, cx| {
                        window.open_sheet(cx, |sheet, _, _| sheet.title("Test Sheet"))
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
        .flex_1()
        .h_full()
        .border_1()
        .border_color(cx.theme().border)
        .rounded_lg()
        .p_4()
        .w_full()
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
