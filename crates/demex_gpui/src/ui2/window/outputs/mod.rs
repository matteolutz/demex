use std::ops::Range;

use demex_core::command::parser::nodes::action::Action;
use demex_dmx::DemexDmxOutputConfig;
use gpui::{
    App, AppContext, Context, Entity, InteractiveElement, ParentElement, Render, SharedString,
    Styled, Subscription, UniformListScrollHandle, WindowBounds, div, prelude::FluentBuilder, size,
    uniform_list,
};
use gpui_component::{
    ActiveTheme, Disableable, IconName, StyledExt,
    alert::Alert,
    button::{Button, ButtonVariants},
    h_flex,
    menu::DropdownMenu,
    scroll::ScrollableElement,
    switch::Switch,
    text::Text,
    v_flex,
};

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::{
        ext::GpuiContextExtension,
        window::outputs::{add_debug::AddDebugOutputWindow, add_serial::AddSerialOutputWindow},
        wm::{
            WindowManager,
            edit_window::{EditWindowDelegate, WindowManagerExtension},
        },
    },
};

mod add_debug;
mod add_serial;

mod actions {
    use gpui::{App, KeyBinding, actions};

    pub const CONTEXT: &str = "demex-output-config";

    actions!(output_config, [AddUsbSerial, AddArtNet, AddDebug, Escape]);

    pub fn init(cx: &mut App) {
        cx.bind_keys([KeyBinding::new("escape", Escape, Some(CONTEXT))]);
    }
}

pub(super) fn init(cx: &mut App) {
    actions::init(cx);
}

pub struct OutputsConfigWindow {
    outputs: Entity<Vec<DemexDmxOutputConfig>>,

    output_list_scroll_handle: UniformListScrollHandle,

    _subscriptions: Vec<Subscription>,
}

impl OutputsConfigWindow {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let patch = DemexUiState::patch(cx);
        let outputs = cx.new(|cx| patch.read(cx).output_configs().to_vec());

        let _subscriptions = vec![
            cx.observe(&patch, |this, patch, cx| {
                this.outputs.update(cx, |outputs, cx| {
                    *outputs = patch.read(cx).output_configs().to_vec();
                    cx.notify();
                });
            }),
            cx.observe_and_notify(&outputs),
        ];

        Self {
            outputs,
            output_list_scroll_handle: UniformListScrollHandle::new(),
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

    fn handle_delete_output(
        &mut self,
        idx: usize,
        _window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        self.update_outputs_and_notify(cx, |outputs, cx| {
            outputs.remove(idx);
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
        let outputs = self.outputs.clone();
        let is_edited = self.is_edited(cx);

        v_flex()
            .key_context(actions::CONTEXT)
                    .on_action(cx.listener(|this, _: &actions::Escape, _, cx| {
                        this.close(cx);
                    }))
                    .on_action(|_: &actions::AddUsbSerial, window, cx| {
                        WindowManager::open_edit_window::<AddSerialOutputWindow>(cx, |cx| AddSerialOutputWindow::new(window, cx));
                    })
                    .on_action(|_: &actions::AddDebug, window, cx| {
                        WindowManager::open_edit_window::<AddDebugOutputWindow>(cx, |cx| AddDebugOutputWindow::new(window, cx));
                    })
                    .size_full()
                    .p_4()
                    .gap_2()
                    .child(div().text_xl().font_semibold().child("Outputs"))
                    .child(Alert::warning(
                        "restart-required",
                        "Changes to the output configuration require\
                        a restart of demex to take effect.",
                    ))
                    .child(
                        h_flex().w_full().justify_end().child(
                            Button::new("save")
                                .label("Save")
                                .disabled(!is_edited)
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    if this.is_edited(cx) {
                                        this.handle_save(window, cx);
                                        this.set_edited(false, cx);
                                    }
                                })),
                        ),
                    )
                    .child(
                        v_flex()
                            .size_full()
                            .gap_4()
                            .mt_4()
                            .overflow_hidden()
                            .vertical_scrollbar(&self.output_list_scroll_handle)
                            .child(Button::new("add").label("Add")
                                .disabled(is_edited)
                                .when(is_edited, |button| button.tooltip("Please save your changes to add a new output"))
                                .when(!is_edited, |button| button.tooltip("Add a new output"))
                                .dropdown_menu(|menu, _, _| {
                                menu.menu("USB Serial", Box::new(actions::AddUsbSerial))
                                    .menu("ArtNet", Box::new(actions::AddArtNet))
                                    .menu("Debug", Box::new(actions::AddDebug))
                            }))
                            .child(
                                uniform_list(
                                    "outputs",
                                    outputs.read(cx).len(),
                                    cx.processor(move |_, range: Range<usize>, _, cx| {
                                        outputs
                                            .read(cx)
                                            .iter()
                                            .skip(range.start)
                                            .take(range.end - range.start)
                                            .enumerate()
                                            .map(|(idx, o)| {
                                                div()
                                                    .w_full()
                                                    .h_24()
                                                    .py_2()
                                                    .child(
                                                h_flex()
                                                    .w_full()
                                                    .h_full()
                                                    .p_4()
                                                    .border_1()
                                                    .border_color(cx.theme().border)
                                                    .rounded_lg()
                                                    .gap_4()
                                                    .child(
                                                        Switch::new(("output-enable", idx))
                                                            .checked(!o.is_disabled())
                                                            .on_click({
                                                                cx.listener(move |this, is_enabled: &bool, _, cx| {
                                                                    this.update_outputs_and_notify(
                                                                        cx,
                                                                        |outputs, _| {
                                                                            outputs[idx]
                                                                                .set_disabled(
                                                                                    !*is_enabled,
                                                                                )
                                                                        },
                                                                    );
                                                                })
                                                            }),
                                                    )
                                                    .child(
                                                        v_flex()
                                                            .gap_0()
                                                            .overflow_hidden()
                                                            .child(
                                                                div()
                                                                    .text_lg()
                                                                    .font_semibold()
                                                                    .child(o.name().to_string()),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_sm()
                                                                    .text_color(cx.theme().muted)
                                                                    .child(Text::String(
                                                                        o.to_string().into(),
                                                                    )),
                                                            ),
                                                    )
                                                    .child(
                                                        h_flex().flex_1().gap_2().justify_end()
                                                            .child(Button::new(("edit", idx)).ghost().icon(IconName::Settings2))
                                                            .child(Button::new(("delete", idx)).ghost().icon(IconName::Close).on_click(
                                                                cx.listener(move |this, _, window, cx| this.handle_delete_output(idx, window, cx))
                                                            ))
                                                    )
                                                )
                                            })
                                            .collect()
                                    }),
                                )
                                .size_full()
                                .track_scroll(self.output_list_scroll_handle.clone()),
                            ),
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
        Some(WindowBounds::centered(size(700.0.into(), 500.0.into()), cx))
    }
}
