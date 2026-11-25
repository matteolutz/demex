use demex_core::command::parser::nodes::action::Action;
use demex_dmx::{
    DemexDmxOutputConfig, DemexDmxOutputConfigData,
    serial::{UsbPortInfo, UsbSerialOutputConfig, available_usb_ports, find_usb_port_by_id},
};
use demex_headless::id::DemexProtoDeviceId;
use gpui::{
    AppContext, ClickEvent, Context, Entity, ParentElement, Render, Styled, Window, WindowBounds,
    div, size,
};
use gpui_component::{
    IconName,
    button::{Button, ButtonVariants},
    form::{field, v_form},
    h_flex,
    input::{InputEvent, InputState, NumberInput},
    notification::Notification,
    select::{Select, SelectEvent, SelectItem, SelectState},
    switch::Switch,
};
use itertools::Itertools;

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::{
        utils::WithId,
        wm::{app::WindowManagerAppExt, edit_window::EditWindowDelegate},
    },
};

impl SelectItem for WithId<UsbPortInfo, (u16, u16)> {
    type Value = (u16, u16);

    fn title(&self) -> gpui::SharedString {
        self.inner().to_string().into()
    }

    fn value(&self) -> &Self::Value {
        self.id()
    }
}

pub struct AddSerialOutputWindow {
    selected_output_device: Entity<SelectState<Vec<WithId<UsbPortInfo, (u16, u16)>>>>,
    universe_input_state: Entity<InputState>,
    enable_rts: bool,

    _subscriptions: Vec<gpui::Subscription>,
}

impl AddSerialOutputWindow {
    pub fn new(window: &mut Window, cx: &mut gpui::Context<Self>) -> Self {
        let selected_output_device = cx.new(|cx| {
            SelectState::new(
                available_usb_ports()
                    .into_iter()
                    .map(WithId::map(|port: &demex_dmx::serial::UsbPortInfo| {
                        port.id()
                    }))
                    .map_into()
                    .collect(),
                None,
                window,
                cx,
            )
        });

        let universe_input_state = cx.new(|cx| InputState::new(window, cx).placeholder("Universe"));

        let _subscriptions = vec![
            cx.subscribe(
                &selected_output_device,
                |this, _, _: &SelectEvent<Vec<WithId<UsbPortInfo, (u16, u16)>>>, cx| {
                    this.set_edited(true, cx);
                },
            ),
            cx.subscribe(
                &universe_input_state,
                |this, _, evt: &InputEvent, cx| match evt {
                    InputEvent::Change => this.set_edited(true, cx),
                    _ => {}
                },
            ),
        ];

        Self {
            selected_output_device,
            universe_input_state,
            enable_rts: false,
            _subscriptions,
        }
    }

    pub fn submit(&self, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(selected_output) = self.selected_output_device.read(cx).selected_value() else {
            return;
        };

        let Some(port) = find_usb_port_by_id(*selected_output) else {
            cx.update_wm(|wm, cx| {
                wm.push_notifcation(Notification::error("USB device not found"), cx)
            });
            return;
        };

        let Ok(universe) = self.universe_input_state.read(cx).value().parse::<u16>() else {
            return;
        };

        let mut output_configs = DemexUiState::patch(cx).read(cx).output_configs().to_vec();
        output_configs.push(DemexDmxOutputConfig::new(
            DemexDmxOutputConfigData::UsbSerial(UsbSerialOutputConfig {
                usb_port: port,
                enable_rts: self.enable_rts,
                universe,
            }),
            DemexProtoDeviceId::Controller,
        ));

        DemexEngineHandler::engine(cx).exec_ui(Action::UpdateOutputConfigs(output_configs.clone()));

        self.discard_and_close(cx);
    }

    fn handle_refresh_devices(
        &mut self,
        _: &ClickEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.selected_output_device.update(cx, |state, cx| {
            state.set_items(
                available_usb_ports()
                    .into_iter()
                    .map(WithId::map(|port: &demex_dmx::serial::UsbPortInfo| {
                        port.id()
                    }))
                    .map_into()
                    .collect(),
                window,
                cx,
            );
            cx.notify();
        });
        cx.notify();
    }
}

impl Render for AddSerialOutputWindow {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div().size_full().p_4().child(
            v_form()
                .child(
                    field().required(true).label("Device").child(
                        h_flex()
                            .gap_1()
                            .child(Select::new(&self.selected_output_device).flex_1())
                            .child(
                                Button::new("refresh-devices")
                                    .ghost()
                                    .icon(IconName::Undo)
                                    .tooltip("Refresh device list")
                                    .on_click(cx.listener(Self::handle_refresh_devices)),
                            ),
                    ),
                )
                .child(
                    field()
                        .required(true)
                        .label("Universe")
                        .child(NumberInput::new(&self.universe_input_state)),
                )
                .child(
                    field().label("Enable RTS").child(
                        Switch::new("enable_rts")
                            .checked(self.enable_rts)
                            .on_click(cx.listener(|this, val, _, cx| {
                                this.enable_rts = *val;
                                this.set_edited(true, cx);
                                cx.notify();
                            })),
                    ),
                )
                .child(
                    field().label_indent(false).child(
                        Button::new("submit")
                            .primary()
                            .child("Add")
                            .on_click(cx.listener(|this, _, window, cx| this.submit(window, cx))),
                    ),
                ),
        )
    }
}

impl EditWindowDelegate for AddSerialOutputWindow {
    fn window_title(
        &self,
        _window: &mut gpui::Window,
        _cx: &gpui::App,
    ) -> impl Into<gpui::SharedString> {
        "Add USB Serial Output"
    }

    fn should_have_save_button(_cx: &gpui::App) -> bool
    where
        Self: Sized,
    {
        false
    }

    fn window_bounds(cx: &mut gpui::App) -> Option<gpui::WindowBounds> {
        Some(WindowBounds::centered(size(500.0.into(), 300.0.into()), cx))
    }

    fn handle_save(&self, _window: &mut gpui::Window, _cx: &mut gpui::App) {}

    fn handle_discard(&self, _window: &mut gpui::Window, _cx: &mut gpui::App) {}
}
