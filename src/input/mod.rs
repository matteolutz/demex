use device::{DemexInputDevice, DemexInputDeviceConfig};
use error::DemexInputDeviceError;
use message::DemexInputDeviceMessage;

use crate::{
    fixture::{
        handler::FixtureHandler, patch::Patch, presets::PresetHandler, selection::FixtureSelection,
        timing::TimingHandler, updatables::UpdatableHandler,
    },
    input::encoder::handle_global_encoder_change,
    lexer::token::Token,
    parser::{
        error::ParseError,
        expected::ExpectedParseSlice,
        nodes::{action::queue::ActionQueue, fixture_selector::FixtureSelectorContext},
    },
    ui::context::EncoderChannels,
};

pub mod button;
pub mod device;
pub mod encoder;
pub mod error;
pub mod fader;
pub mod message;
pub mod midi;
pub mod profile;
pub mod timecode;

pub trait DemexInputDeviceProfile: std::fmt::Debug {
    fn update_out(
        &mut self,
        device_config: &DemexInputDeviceConfig,
        preset_handler: &PresetHandler,
        updatable_handler: &UpdatableHandler,
        timing_handler: &TimingHandler,
        global_fixture_selection: &Option<FixtureSelection>,
    ) -> Result<(), DemexInputDeviceError>;

    fn poll(&self) -> Result<Vec<DemexInputDeviceMessage>, DemexInputDeviceError>;

    fn is_enabled(&self) -> bool;
}

#[derive(Default, Debug)]
pub struct DemexInputDeviceHandler {
    devices: Vec<DemexInputDevice>,
}

impl DemexInputDeviceHandler {
    pub fn new(devices: Vec<DemexInputDevice>) -> Self {
        Self { devices }
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
                            button.handle_press(
                                fixture_handler,
                                preset_handler,
                                updatable_handler,
                                timing_handler,
                                patch,
                                fixture_selector_context.clone(),
                                macro_exec_cue,
                                global_fixture_selection,
                                command_input,
                            )?;
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
                        button.handle_release(
                            fixture_handler,
                            preset_handler,
                            updatable_handler,
                        )?;
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
                            fader.handle_change(
                                value,
                                fixture_handler,
                                preset_handler,
                                updatable_handler,
                                timing_handler,
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
                            )?;
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
                        handle_global_encoder_change(
                            encoder_idx,
                            value,
                            fixture_selector_context.clone(),
                            fixture_handler,
                            encoder_channels,
                            patch,
                        )
                    }
                }
            }
        }

        for device in &mut self.devices {
            if !device.profile().is_enabled() {
                continue;
            }

            device.profile.update_out(
                &device.config,
                preset_handler,
                updatable_handler,
                timing_handler,
                global_fixture_selection,
            )?;
        }

        Ok(())
    }
}
