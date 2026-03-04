use device::{DemexInputDevice, DemexInputDeviceConfig};
use error::DemexInputDeviceError;
use message::DemexInputDeviceMessage;

use crate::{
    EncoderChannels,
    command::parser::{
        error::ParseError,
        expected::ExpectedParseSlice,
        nodes::{action::queue::ActionQueue, fixture_selector::FixtureSelectorContext},
    },
    event::list::DemexEventList,
    input::{
        control::{
            DemexInputDeviceControlAssignment, DemexInputDeviceControlUnassignment,
            button::DemexInputButton, encoder::DemexInputEncoder, fader::DemexInputFader,
        },
        event::{DemexInputDeviceConfigExt, DemexInputDeviceControlUpdate},
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

pub trait DemexInputDeviceProfile: 'static + Send + std::fmt::Debug {
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
        events: &[DemexInputDeviceControlUpdate],
    ) -> Result<(), DemexInputDeviceError>;

    fn poll(&mut self) -> Result<Vec<DemexInputDeviceMessage>, DemexInputDeviceError>;

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

    pub fn device_configs(&self) -> impl Iterator<Item = &DemexInputDeviceConfig> {
        self.devices.iter().map(|dev| &dev.config)
    }

    pub fn assign(
        &mut self,
        assignment: DemexInputDeviceControlAssignment,
    ) -> Result<(), DemexInputDeviceError> {
        match assignment {
            DemexInputDeviceControlAssignment::Button {
                device_idx,
                button_id,
                assignment,
            } => {
                let device = self.device_mut(device_idx)?;
                device.assign_button(button_id, assignment)
            }
            DemexInputDeviceControlAssignment::Fader {
                device_idx,
                fader_id,
                assignment,
            } => {
                let device = self.device_mut(device_idx)?;
                device.assign_fader(fader_id, assignment)
            }
        }
    }

    pub fn unassign(
        &mut self,
        unassignment: DemexInputDeviceControlUnassignment,
    ) -> Result<(), DemexInputDeviceError> {
        match unassignment {
            DemexInputDeviceControlUnassignment::Button {
                device_idx,
                button_id,
            } => {
                let device = self.device_mut(device_idx)?;
                device.unassign_button(button_id)
            }
            DemexInputDeviceControlUnassignment::Fader {
                device_idx,
                fader_id,
            } => {
                let device = self.device_mut(device_idx)?;
                device.unassign_fader(fader_id)
            }
        }
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
        // first poll all events
        for (device_idx, device) in self
            .devices
            .iter_mut()
            .filter(|device| device.profile().is_enabled())
            .enumerate()
        {
            // make this immut
            for device_msg in device
                .profile_mut()
                .poll()
                .inspect_err(|err| {
                    log::warn!(
                        "Failed to poll events for device {:?}: {}",
                        device.profile,
                        err
                    )
                })
                .unwrap_or_default()
            {
                match device_msg {
                    DemexInputDeviceMessage::ButtonPressed(button_id) => {
                        let parse_error = parse_command_input();

                        if parse_error.as_ref().is_some_and(|err| {
                            err.was_expected(ExpectedParseSlice::ButtonId { is_unassign: true })
                        }) {
                            append_to_command(format!("{}.{}", device_idx, button_id));
                            continue;
                        }

                        let button = device.config().buttons().get(&button_id);

                        if let Some(button) = button {
                            button.handle_press(action_queue)?;
                        } else if parse_error.is_some_and(|err| {
                            err.was_expected(ExpectedParseSlice::ButtonId { is_unassign: false })
                        }) {
                            append_to_command(format!("{}.{}", device_idx, button_id));
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
                            append_to_command(format!("{}.{}", device_idx, fader_id));
                        }
                    }
                    DemexInputDeviceMessage::FaderValueChanged(fader_id, value) => {
                        let parse_error = parse_command_input();

                        if parse_error.as_ref().is_some_and(|err| {
                            err.was_expected(ExpectedParseSlice::FaderId { is_unassign: true })
                        }) {
                            append_to_command(format!("{}.{}", device_idx, fader_id));
                            continue;
                        }

                        let fader = device.config().faders().get(&fader_id);

                        if let Some(fader) = fader {
                            fader.handle_change(value, action_queue)?;
                        } else if parse_error.is_some_and(|err| {
                            err.was_expected(ExpectedParseSlice::FaderId { is_unassign: false })
                        }) {
                            append_to_command(format!("{}.{}", device_idx, fader_id));
                        }
                    }
                    DemexInputDeviceMessage::FaderValuesChanged(fader_values) => {
                        for (fader_id, value) in fader_values {
                            let fader = device
                                .config()
                                .faders()
                                .get(&fader_id)
                                .ok_or(DemexInputDeviceError::ButtonNotFound(fader_id))?;

                            fader.handle_change(value, action_queue)?;
                        }
                    }
                    DemexInputDeviceMessage::Timecode(_timecode_packet) => {
                        todo!()
                    }
                    DemexInputDeviceMessage::TimecodeQuarterFrame { piece: _ } => {
                        todo!()
                    }
                    DemexInputDeviceMessage::GlobalEncoderClick(_) => {}
                    DemexInputDeviceMessage::GlobalEncoderValueChanged {
                        encoder_idx,
                        value: _,
                    } => {
                        let _encoder = DemexInputEncoder::GlobalEncoder { encoder_idx };
                        todo!()
                    }
                };
            }
        }

        for device in self
            .devices
            .iter_mut()
            .filter(|device| device.profile().is_enabled())
        {
            let args = DemexInputDeviceUpdateArgs {
                device_config: &device.config,
                fixture_selector_context,
                patch,
                encoder_channels,
            };

            let events = device.config.map_events(args.clone(), event_list.events());

            if let Err(err) = device.profile.handle_events(&events) {
                log::error!(
                    "Error handling events for device {:?}: {}",
                    device.profile,
                    err
                );
            };

            if let Err(err) = device.profile.tick(args) {
                log::error!("Error ticking device {:?}: {}", device.profile, err);
            };
        }

        Ok(())
        /*
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

        */
    }
}
