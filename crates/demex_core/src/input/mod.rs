use device::{DemexInputDevice, DemexInputDeviceConfig};
use error::DemexInputDeviceError;
use message::DemexInputDeviceMessage;

use crate::{
    EncoderChannels,
    command::{
        lexer::token::Token,
        parser::{
            error::ParseError,
            expected::ExpectedParseSlice,
            nodes::{action::queue::ActionQueue, fixture_selector::FixtureSelectorContext},
        },
    },
    event::list::DemexEventList,
    input::{
        control::{
            DemexInputDeviceControlTrait, button::DemexInputButton, encoder::DemexInputEncoder,
            fader::DemexInputFader,
        },
        event::DemexInputDeviceControlUpdate,
    },
    patch::Patch,
};

pub mod control;
pub mod device;
pub mod encoder;
pub mod error;
pub mod event;
pub mod message;
pub mod midi;
pub mod profile;
pub mod timecode;

#[derive(Debug, Clone)]
pub struct DemexInputDeviceUpdateArgs<'a> {
    pub device_config: &'a DemexInputDeviceConfig,

    pub fixture_selector_context: FixtureSelectorContext<'a>,
    pub patch: &'a Patch,
    pub encoder_channels: Option<&'a EncoderChannels>,
}

pub trait DemexInputDeviceProfile: std::fmt::Debug {
    fn handle_button_assign(
        &mut self,
        _button_id: u32,
        _button: &DemexInputButton,
    ) -> Result<(), DemexInputDeviceError> {
        Ok(())
    }

    fn handle_button_unassign(
        &mut self,
        _button_id: u32,
        _button: &DemexInputButton,
    ) -> Result<(), DemexInputDeviceError> {
        Ok(())
    }

    fn handle_fader_assign(
        &mut self,
        _fader_id: u32,
        _fader: &DemexInputFader,
    ) -> Result<(), DemexInputDeviceError> {
        Ok(())
    }

    fn handle_fader_unassign(
        &mut self,
        _fader_id: u32,
        _fader: &DemexInputFader,
    ) -> Result<(), DemexInputDeviceError> {
        Ok(())
    }

    fn num_global_encoders(&self) -> u32 {
        0
    }

    fn tick(&mut self, args: DemexInputDeviceUpdateArgs) -> Result<(), DemexInputDeviceError>;
    fn handle_events(
        &mut self,
        args: DemexInputDeviceUpdateArgs,
        events: &[DemexInputDeviceControlUpdate],
    ) -> Result<(), DemexInputDeviceError>;

    fn poll(&mut self) -> Result<Vec<DemexInputDeviceMessage>, DemexInputDeviceError>;

    fn is_enabled(&self) -> bool;
}

#[derive(Default, Debug)]
pub struct DemexInputDeviceHandler {
    devices: Vec<DemexInputDevice>,
    has_initialized: bool,
}

impl DemexInputDeviceHandler {
    pub fn new(devices: Vec<DemexInputDevice>) -> Self {
        Self {
            devices,
            has_initialized: false,
        }
    }

    pub fn device_mut(
        &mut self,
        idx: usize,
    ) -> Result<&mut DemexInputDevice, DemexInputDeviceError> {
        self.devices
            .get_mut(idx)
            .ok_or(DemexInputDeviceError::InputDeviceIdxNotFound(idx))
    }

    pub fn devices(&self) -> &Vec<DemexInputDevice> {
        &self.devices
    }

    pub fn update(
        &mut self,
        patch: &Patch,
        fixture_selector_context: FixtureSelectorContext,
        action_queue: &mut ActionQueue,

        append_to_command: impl Fn(String),
        parse_command_input: impl Fn() -> Option<ParseError>,

        encoder_channels: Option<&EncoderChannels>,
        event_list: &mut DemexEventList,
    ) -> Result<(), DemexInputDeviceError> {
        todo!()
        /*
            for (device_idx, device) in self.devices.iter_mut().enumerate() {
                if !device.profile().is_enabled() {
                    continue;
                }

                for device_message in device.profile_mut().poll()? {
                    match device_message {
                        DemexInputDeviceMessage::ButtonPressed(button_id) => {
                            let parse_error = parse_command_input();

                            if parse_error.as_ref().is_some_and(|err| {
                                err.was_expected(ExpectedParseSlice::ButtonId { is_unassign: true })
                            }) {
                                append_to_command(format!(" {}.{}", device_idx, button_id));
                                continue;
                            }

                            let button = device.config().buttons().get(&button_id);

                            if let Some(button) = button {
                                button.handle_press(action_queue)?;
                            } else if parse_error.is_some_and(|err| {
                                err.was_expected(ExpectedParseSlice::ButtonId { is_unassign: false })
                            }) {
                                append_to_command(format!(" {}.{}", device_idx, button_id));
                            }
                        }
                        DemexInputDeviceMessage::ButtonReleased(button_id) => {
                            let button = device
                                .config()
                                .buttons()
                                .get(&button_id)
                                .ok_or(DemexInputDeviceError::ButtonNotFound(button_id))?;

                            button.handle_release(action_queue)?;
                        }

                        DemexInputDeviceMessage::FaderTouch(fader_id) => {
                            let parse_error = parse_command_input();

                            if parse_error.as_ref().is_some_and(|err| {
                                err.was_expected(ExpectedParseSlice::FaderId { is_unassign: true })
                                    || err.was_expected(ExpectedParseSlice::FaderId {
                                        is_unassign: false,
                                    })
                            }) {
                                append_to_command(format!(" {}.{}", device_idx, fader_id));
                            }
                        }
                        DemexInputDeviceMessage::FaderValueChanged(fader_id, value) => {
                            let parse_error = parse_command_input();

                            if parse_error.as_ref().is_some_and(|err| {
                                err.was_expected(ExpectedParseSlice::FaderId { is_unassign: true })
                            }) {
                                append_to_command(format!(" {}.{}", device_idx, fader_id));
                                continue;
                            }

                            let fader = device.config().faders().get(&fader_id);

                            if let Some(fader) = fader {
                                fader.handle_change(
                                    value,
                                    fixture_handler,
                                    preset_handler,
                                    updatable_handler,
                                    timing_handler,
                                    event_list,
                                )?;
                            } else if parse_error.is_some_and(|err| {
                                err.was_expected(ExpectedParseSlice::FaderId { is_unassign: false })
                            }) {
                                command_input.extend_from_slice(&[Token::FloatingPoint(
                                    0.0,
                                    (device_idx as u32, fader_id),
                                )]);
                            }
                        }
                        DemexInputDeviceMessage::FaderValuesChanged(fader_values) => {
                            for (fader_id, value) in fader_values {
                                let fader = device
                                    .config()
                                    .faders()
                                    .get(&fader_id)
                                    .ok_or(DemexInputDeviceError::ButtonNotFound(fader_id))?;

                                fader.handle_change(
                                    value,
                                    fixture_handler,
                                    preset_handler,
                                    updatable_handler,
                                    timing_handler,
                                    event_list,
                                )?;
                            }
                        }
                        DemexInputDeviceMessage::Timecode(timecode_packet) => {
                            timing_handler.handle_timecode_packet(timecode_packet)
                        }
                        DemexInputDeviceMessage::TimecodeQuarterFrame { piece } => {
                            timing_handler.handle_timecode_quarter_frame(piece)
                        }
                        DemexInputDeviceMessage::GlobalEncoderClick(_) => {}
                        DemexInputDeviceMessage::GlobalEncoderValueChanged { encoder_idx, value } => {
                            let encoder = DemexInputEncoder::GlobalEncoder { encoder_idx };
                            encoder.handle_change(
                                value,
                                fixture_selector_context.clone(),
                                fixture_handler,
                                encoder_channels,
                                preset_handler,
                                updatable_handler,
                                timing_handler,
                                patch,
                                event_list,
                            )?;
                        }
                    };
                }
            }

            for device in &mut self.devices {
                if !device.profile().is_enabled() {
                    continue;
                }

                let args = DemexInputDeviceUpdateArgs {
                    device_config: &device.config,
                    fixture_selector_contxt: fixture_selector_context.clone(),
                    patch,
                    encoder_channels,
                };

                let mut device_events = vec![];

                if !self.has_initialized {
                    for (id, button) in device.config.buttons() {
                        device_events.push(DemexInputDeviceControlUpdate::Button {
                            id: *id,
                            button,
                            update: button.initial_state(args.clone())?,
                        });
                    }

                    for (id, fader) in device.config.faders() {
                        device_events.push(DemexInputDeviceControlUpdate::Fader {
                            id: *id,
                            fader,
                            update: fader.initial_state(args.clone())?,
                        });
                    }

                    for (id, encoder) in device.config.encoders() {
                        device_events.push(DemexInputDeviceControlUpdate::Encoder {
                            id: *id,
                            encoder,
                            update: encoder.initial_state(args.clone())?,
                        });
                    }

                    for encoder_idx in 0..device.profile.num_global_encoders() {
                        let encoder = DemexInputEncoder::GlobalEncoder { encoder_idx };
                        device_events.push(DemexInputDeviceControlUpdate::GlobalEncoder {
                            id: encoder_idx,
                            update: encoder.initial_state(args.clone())?,
                        });
                    }
                }

                for event in event_list.events() {
                    for (id, button) in device.config.buttons() {
                        if let Some(update) = button.should_update(args.clone(), event).ok().flatten() {
                            device_events.push(DemexInputDeviceControlUpdate::Button {
                                button,
                                update,
                                id: *id,
                            });
                        }
                    }

                    for (id, fader) in device.config.faders() {
                        if let Some(update) = fader.should_update(args.clone(), event).ok().flatten() {
                            device_events.push(DemexInputDeviceControlUpdate::Fader {
                                fader,
                                update,
                                id: *id,
                            });
                        }
                    }

                    for (id, encoder) in device.config.encoders() {
                        if let Some(update) = encoder.should_update(args.clone(), event).ok().flatten()
                        {
                            device_events.push(DemexInputDeviceControlUpdate::Encoder {
                                encoder,
                                update,
                                id: *id,
                            });
                        }
                    }

                    for encoder_idx in 0..device.profile().num_global_encoders() {
                        let encoder = DemexInputEncoder::GlobalEncoder { encoder_idx };
                        if let Some(update) = encoder.should_update(args.clone(), event).ok().flatten()
                        {
                            device_events.push(DemexInputDeviceControlUpdate::GlobalEncoder {
                                update,
                                id: encoder_idx,
                            });
                        }
                    }
                }

                device.profile.handle_events(args.clone(), &device_events)?;
                device.profile.tick(args)?;
            }

            if !self.has_initialized {
                self.has_initialized = true;
            }

            Ok(())
        */
    }
}
