use demex_core::command::parser::nodes::action::Action;
use demex_dmx::DemexDmxOutputConfig;
use gpui::{
    AppContext, Bounds, Context, Entity, ParentElement, Render, SharedString, Styled, Subscription,
    div, size,
};
use gpui_component::{
    ActiveTheme, Disableable, StyledExt, alert::Alert, button::Button, h_flex,
    scroll::ScrollableElement, switch::Switch, text::Text, v_flex,
};

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::{
        ext::GpuiContextExtension, titlebar::DemexTitleBar, wm::edit_window::EditWindowDelegate,
    },
};

pub struct OutputsConfigWindow {
    titlebar: Entity<DemexTitleBar>,
    outputs: Entity<Vec<DemexDmxOutputConfig>>,
    _subscriptions: Vec<Subscription>,
}

impl OutputsConfigWindow {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let patch = DemexUiState::patch(cx);

        let _subscriptions = vec![cx.observe_and_notify(&patch)];

        let outputs = cx.new(|cx| patch.read(cx).output_configs().to_vec());

        Self {
            titlebar: cx.new(|_| DemexTitleBar::settings("Outputs")),
            outputs,
            _subscriptions,
        }
    }

    fn update_outputs_and_notify(
        &mut self,
        cx: &mut Context<Self>,
        update: impl FnOnce(&mut Vec<DemexDmxOutputConfig>, &mut Context<Vec<DemexDmxOutputConfig>>),
    ) {
        self.outputs.update(cx, |outputs, cx| {
            update(outputs, cx);
            cx.notify();
        });
        self.set_edited(true, cx);
    }
}

impl Render for OutputsConfigWindow {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let outputs = self.outputs.read(cx);
        let is_edited = self.is_edited(cx);

        v_flex()
            .size_full()
            .child(self.titlebar.clone())
            .child(
                v_flex().p_4().gap_2().size_full()
                    .overflow_y_scrollbar()
                    .child(div().text_xl().font_semibold().child("Outputs"))
                    .child(Alert::warning(
                        "restart-required",
                        "Changes to the output configuration require a restart of demex to take effect.",
                    ))
                    .child(h_flex().w_full().justify_end().child(Button::new("save").label("Save").disabled(!is_edited).on_click(cx.listener(move |this, _, window, cx| {
                        if this.is_edited(cx) {
                            this.handle_save(window, cx);
                            this.set_edited(false, cx);
                        }
                    }))))
                    .child(
                        v_flex().size_full().gap_4().mt_4()
                            .children(outputs.iter().enumerate().map(|(idx, o)|
                                h_flex()
                                    .p_4()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_lg()
                                    .gap_4()
                                    .child(Switch::new(("output-enable", idx)).checked(!o.is_disabled()).on_click(cx.listener(move |this, is_enabled: &bool, _, cx| {
                                        this.update_outputs_and_notify(cx, |outputs, _| outputs[idx].set_disabled(!*is_enabled));
                                    })))
                                    .child(
                                        v_flex()
                                            .gap_0()
                                            .overflow_hidden()
                                            .child(div().text_lg().font_semibold().child(o.name().to_string()))
                                            .child(div().text_sm().text_color(cx.theme().muted).child(Text::String(o.to_string().into())))
                                    )
                                )
                            )
                            .child(Button::new("add").label("Add"))
                        )
                )
    }
}

impl EditWindowDelegate for OutputsConfigWindow {
    fn window_title(&self, _window: &mut gpui::Window, _cx: &gpui::App) -> impl Into<SharedString> {
        "Output Config"
    }

    fn handle_save(&self, _window: &mut gpui::Window, cx: &mut gpui::App) {
        let output_configs = self.outputs.read(cx).clone();
        DemexEngineHandler::engine(cx).exec_ui(Action::UpdateOutputConfigs(output_configs.clone()));
    }

    fn handle_discard(&self, _window: &mut gpui::Window, _cx: &mut gpui::App) {
        // do nothing
    }

    fn window_bounds(cx: &mut gpui::App) -> Option<gpui::WindowBounds> {
        Some(gpui::WindowBounds::Windowed(Bounds::centered(
            None,
            size(500.0.into(), 300.0.into()),
            cx,
        )))
    }
}
