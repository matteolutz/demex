use gpui::{
    App, AppContext, Context, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement,
    Render, Styled, Subscription, Window, div, prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, StyledExt,
    button::Button,
    chart::AreaChart,
    dock::{Panel, PanelEvent, register_panel},
    scroll::ScrollbarAxis,
    v_flex,
};

use crate::{
    engine::state::DemexUiState,
    ui2::{
        ext::GpuiContextExtension,
        window::outputs::OutputsConfigWindow,
        wm::{WindowManager, edit_window::WindowManagerExtension},
    },
};

const PERFORMANCE_PANEL_NAME: &str = "demex-performance";

pub(super) fn register(cx: &mut App) {
    register_panel(cx, PERFORMANCE_PANEL_NAME, |_, _, _, window, cx| {
        Box::new(cx.new(|cx| PerformancePanel::new(window, cx)))
    });
}

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
        PERFORMANCE_PANEL_NAME
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

        div().w_full().scrollable(ScrollbarAxis::Both).child(
            v_flex()
                .w_full()
                .min_w_64()
                .p_4()
                .gap_4()
                .justify_center()
                .child(Button::new("test").label("Test").on_click(|_, _, cx| {
                    cx.defer(|cx| {
                        WindowManager::open_edit_window::<OutputsConfigWindow>(cx, |cx| {
                            OutputsConfigWindow::new(cx)
                        });
                    });
                }))
                .child(div().font_bold().text_2xl().child("Performance"))
                .children(performance.into_iter().map(|(thread_name, stats)| {
                    div().w_full().child(stats_chart_container(
                        thread_name,
                        stats
                            .current_its()
                            .map(|its| format!("Current: {:.0}its", its)),
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
