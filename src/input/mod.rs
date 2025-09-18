use device::{DemexInputDevice, DemexInputDeviceConfig};
use error::DemexInputDeviceError;
use message::DemexInputDeviceMessage;

use crate::{
    fixture::{
        handler::FixtureHandler, patch::Patch, presets::PresetHandler, selection::FixtureSelection,
        timing::TimingHandler, updatables::UpdatableHandler,
    },
    input::{
        control::{
            button::DemexInputButton, encoder::DemexInputEncoder, fader::DemexInputFader,
            DemexInputDeviceControlTrait,
        },
        event::{DemexInputDeviceControlUpdate, DemexInputDeviceEvent},
    },
    lexer::token::Token,
    parser::{
        error::ParseError,
        expected::ExpectedParseSlice,
        nodes::{action::queue::ActionQueue, fixture_selector::FixtureSelectorContext},
    },
    ui::context::EncoderChannels,
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
    pub fixture_handler: &'a FixtureHandler,
    pub preset_handler: &'a PresetHandler,
    pub updatable_handler: &'a UpdatableHandler,
    pub timing_handler: &'a TimingHandler,
    pub global_fixture_selection: &'a Option<FixtureSelection>,
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

    fn poll(&self) -> Result<Vec<DemexInputDeviceMessage>, DemexInputDeviceError>;

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

    pub fn update<F>(
        &mut self,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &mut PresetHandler,
        updatable_handler: &mut UpdatableHandler,
        timing_handler: &mut TimingHandler,
        patch: &Patch,
        fixture_selector_context: FixtureSelectorContext,
        macro_exec_cue: &mut ActionQueue,
        global_fixture_selection: &mut Option<FixtureSelection>,
        command_input: &mut Vec<Token>,
        parse_command_input: F,
        encoder_channels: Option<&EncoderChannels>,
        events: &mut Vec<DemexInputDeviceEvent>,
    ) -> Result<(), DemexInputDeviceError>
    where
        F: Fn(&[Token]) -> Option<ParseError>,
    {
        for (device_idx, device) in self.devices.iter().enumerate() {
            if !device.profile().is_enabled() {
                continue;
            }

            for device_message in device.profile().poll()? {
                match device_message {
                    DemexInputDeviceMessage::ButtonPressed(button_id) => {
                        let parse_error = parse_command_input(command_input);

                        if parse_error.as_ref().is_some_and(|err| {
                            err.was_expected(ExpectedParseSlice::ButtonId { is_unassign: true })
                        }) {
                            command_input.extend_from_slice(&[Token::FloatingPoint(
                                0.0,
                                (device_idx as u32, button_id),
                            )]);
                            continue;
                        }

                        let button = device.config().buttons().get(&button_id);

                        if let Some(button) = button {
                            if let Some(event) = button.handle_press(
                                fixture_handler,
                                preset_handler,
                                updatable_handler,
                                timing_handler,
                                patch,
                                fixture_selector_context.clone(),
                                macro_exec_cue,
                                global_fixture_selection,
                                command_input,
                            )? {
                                events.push(event);
                            }
                        } else if parse_error.is_some_and(|err| {
                            err.was_expected(ExpectedParseSlice::ButtonId { is_unassign: false })
                        }) {
                            command_input.extend_from_slice(&[Token::FloatingPoint(
                                0.0,
                                (device_idx as u32, button_id),
                            )]);
                        }
                    }
                    DemexInputDeviceMessage::ButtonReleased(button_id) => {
                        let button = device
                            .config()
                            .buttons()
                            .get(&button_id)
                            .ok_or(DemexInputDeviceError::ButtonNotFound(button_id))?;

                        if let Some(event) = button.handle_release(
                            fixture_handler,
                            preset_handler,
                            updatable_handler,
                        )? {
                            events.push(event);
                        }
                    }

                    DemexInputDeviceMessage::FaderTouch(fader_id) => {
                        let parse_error = parse_command_input(command_input);

                        if parse_error.as_ref().is_some_and(|err| {
                            err.was_expected(ExpectedParseSlice::FaderId { is_unassign: true })
                                || err.was_expected(ExpectedParseSlice::FaderId {
                                    is_unassign: false,
                                })
                        }) {
                            command_input.extend_from_slice(&[Token::FloatingPoint(
                                0.0,
                                (device_idx as u32, fader_id),
                            )]);
                        }
                    }
                    DemexInputDeviceMessage::FaderValueChanged(fader_id, value) => {
                        let parse_error = parse_command_input(command_input);

                        if parse_error.as_ref().is_some_and(|err| {
                            err.was_expected(ExpectedParseSlice::FaderId { is_unassign: true })
                        }) {
                            command_input.extend_from_slice(&[Token::FloatingPoint(
                                0.0,
                                (device_idx as u32, fader_id),
                            )]);
                            continue;
                        }

                        let fader = device.config().faders().get(&fader_id);

                        if let Some(fader) = fader {
                            if let Some(event) = fader.handle_change(
                                value,
                                fixture_handler,
                                preset_handler,
                                updatable_handler,
                                timing_handler,
                            )? {
                                events.push(event);
                            }
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

                            if let Some(event) = fader.handle_change(
                                value,
                                fixture_handler,
                                preset_handler,
                                updatable_handler,
                                timing_handler,
                            )? {
                                events.push(event);
                            }
                        }
                    }
                    DemexInputDeviceMessage::Timecode(timecode_packet) => timing_handler
                        .update_timecode(
                            timecode_packet,
                            fixture_handler,
                            preset_handler,
                            updatable_handler,
                        ),
                    DemexInputDeviceMessage::TimecodeQuarterFrame { piece } => timing_handler
                        .update_timecode_quarter_frame(
                            piece,
                            fixture_handler,
                            preset_handler,
                            updatable_handler,
                        ),
                    DemexInputDeviceMessage::GlobalEncoderClick(_) => {}
                    DemexInputDeviceMessage::GlobalEncoderValueChanged { encoder_idx, value } => {
                        let encoder = DemexInputEncoder::GlobalEncoder { encoder_idx };
                        if let Some(event) = encoder.handle_change(
                            value,
                            fixture_selector_context.clone(),
                            fixture_handler,
                            encoder_channels,
                            preset_handler,
                            updatable_handler,
                            timing_handler,
                            patch,
                        )? {
                            events.push(event);
                        }
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
                fixture_handler,
                preset_handler,
                updatable_handler,
                timing_handler,
                global_fixture_selection,
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
            }

            for event in events.iter() {
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

        events.clear();
        if !self.has_initialized {
            self.has_initialized = true;
        }

        Ok(())
    }
}
