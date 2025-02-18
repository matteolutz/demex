use std::sync::mpsc;

use crate::{
    fixture::updatables::error::UpdatableHandlerError,
    input::{
        button::DemexInputButton, error::DemexInputDeviceError, message::DemexInputDeviceMessage,
        midi::MidiMessage, DemexInputDeviceProfile,
    },
    parser::nodes::fixture_selector::FixtureSelector,
};

use super::DemexInputDeviceProfileType;

// **Ressources**
// https://cdn.inmusicbrands.com/akai/attachments/APC%20mini%20mk2%20-%20Communication%20Protocol%20-%20v1.0.pdf

const APC_MINI_MK_2_NAME: &str = "APC mini mk2 Control";

pub struct ApcMiniMk2InputDeviceProfile {
    rx: mpsc::Receiver<MidiMessage>,
    midi_out: midir::MidiOutputConnection,
    _midi_in: midir::MidiInputConnection<()>,
}

impl std::fmt::Debug for ApcMiniMk2InputDeviceProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApcMiniMk2InputDeviceProfile")
    }
}

impl ApcMiniMk2InputDeviceProfile {
    pub fn new() -> Result<Self, DemexInputDeviceError> {
        let (tx, rx) = mpsc::channel();

        let midi_out = midir::MidiOutput::new("demex-midi-output")
            .map_err(|err| DemexInputDeviceError::MidirError(err.into()))?;

        let out_ports = midi_out.ports();
        let out_port = out_ports
            .iter()
            .find(|p| {
                midi_out
                    .port_name(p)
                    .is_ok_and(|port_name| port_name == APC_MINI_MK_2_NAME)
            })
            .ok_or(DemexInputDeviceError::InputDeviceNotFound(
                APC_MINI_MK_2_NAME.to_owned(),
            ))?;

        let connection = midi_out
            .connect(out_port, APC_MINI_MK_2_NAME)
            .map_err(|err| DemexInputDeviceError::MidirError(err.into()))?;

        let midi_in = midir::MidiInput::new("demex-midi-input")
            .map_err(|err| DemexInputDeviceError::MidirError(err.into()))?;

        let in_ports = midi_in.ports();
        let in_port = in_ports
            .iter()
            .find(|p| {
                midi_in
                    .port_name(p)
                    .is_ok_and(|port_name| port_name == APC_MINI_MK_2_NAME)
            })
            .ok_or(DemexInputDeviceError::InputDeviceNotFound(
                APC_MINI_MK_2_NAME.to_owned(),
            ))
            .unwrap();

        let conn_in = midi_in
            .connect(
                in_port,
                APC_MINI_MK_2_NAME,
                move |_, msg, _| {
                    if let Some(midi_msg) = MidiMessage::from_bytes(msg) {
                        tx.send(midi_msg).unwrap();
                    } else {
                        println!("failed to serialize midi bytes: {:?}", msg);
                    }
                },
                (),
            )
            .map_err(|err| DemexInputDeviceError::MidirError(err.into()))?;

        let mut s = Self {
            rx,
            midi_out: connection,
            _midi_in: conn_in,
        };

        s.init()?;

        Ok(s)
    }

    pub fn init(&mut self) -> Result<(), DemexInputDeviceError> {
        self.midi_out
            .send(&[
                0xF0, 0x47, 0x7F, 0x4F, 0x60, 0x0, 0x04, // akai
                0x0, 0x0, 0x0, 0x1,  // demex version
                0xF7, // end of sysex
            ])
            .unwrap();

        for i in 0..=63 {
            self.midi_out
                .send(
                    &MidiMessage::NoteOn {
                        channel: 6,
                        note_number: i,
                        key_velocity: 0,
                    }
                    .to_bytes(),
                )
                .map_err(|err| DemexInputDeviceError::MidirError(err.into()))?;
        }

        Ok(())
    }

    pub fn get_button_note_number(&self, idx: u32) -> Option<u8> {
        match idx {
            0..=63 => Some(idx as u8),
            100..=107 => Some((idx - 100) as u8 + 0x64),
            200..=207 => Some((idx - 200) as u8 + 0x70),
            300 => Some(0x7A),
            _ => None,
        }
    }

    pub fn get_button_idx(&self, _channel: u8, note_number: u8) -> Option<u32> {
        match note_number {
            0x00..=0x3F => Some(note_number as u32),
            0x64..=0x6B => Some(note_number as u32 - 0x64 + 100),
            0x70..=0x77 => Some(note_number as u32 - 0x70 + 200),
            0x7A => Some(300),
            _ => None,
        }
    }

    pub fn get_fader_idx(&self, channel: u8, control_code: u8) -> Option<u32> {
        if channel != 0 {
            return None;
        }

        match control_code {
            0x30..=0x38 => Some(control_code as u32 - 0x30),
            _ => None,
        }
    }
}

impl DemexInputDeviceProfile for ApcMiniMk2InputDeviceProfile {
    fn update_out(
        &mut self,
        device_config: &crate::input::device::DemexInputDeviceConfig,
        updatable_handler: &crate::fixture::updatables::UpdatableHandler,
        global_fixture_selector: &Option<FixtureSelector>,
    ) -> Result<(), DemexInputDeviceError> {
        for (button_id, button) in device_config.buttons() {
            match button {
                DemexInputButton::ExecutorStartAndNext(id) => {
                    let is_started = updatable_handler
                        .executor(*id)
                        .ok_or(DemexInputDeviceError::UpdatableHandlerError(
                            UpdatableHandlerError::UpdatableNotFound(*id),
                        ))?
                        .is_started();

                    self.midi_out
                        .send(
                            &MidiMessage::NoteOn {
                                channel: if !is_started {
                                    6 // 100% brightness
                                } else {
                                    0xd // blinking 1/8
                                },
                                note_number: self
                                    .get_button_note_number(*button_id)
                                    .ok_or(DemexInputDeviceError::ButtonNotFound(*button_id))?,
                                key_velocity: 21, // green,
                            }
                            .to_bytes(),
                        )
                        .map_err(|err| DemexInputDeviceError::MidirError(err.into()))?;
                }
                DemexInputButton::ExecutorStop(id) => {
                    let is_started = updatable_handler
                        .executor(*id)
                        .ok_or(DemexInputDeviceError::UpdatableHandlerError(
                            UpdatableHandlerError::UpdatableNotFound(*id),
                        ))?
                        .is_started();

                    self.midi_out
                        .send(
                            &MidiMessage::NoteOn {
                                channel: if !is_started {
                                    6 // 100% brightness
                                } else {
                                    10 // pulsing 1/2
                                },
                                note_number: self
                                    .get_button_note_number(*button_id)
                                    .ok_or(DemexInputDeviceError::ButtonNotFound(*button_id))?,
                                key_velocity: 5, // red,
                            }
                            .to_bytes(),
                        )
                        .map_err(|err| DemexInputDeviceError::MidirError(err.into()))?;
                }
                DemexInputButton::ExecutorFlash(id) => {
                    let is_started = updatable_handler
                        .executor(*id)
                        .ok_or(DemexInputDeviceError::UpdatableHandlerError(
                            UpdatableHandlerError::UpdatableNotFound(*id),
                        ))?
                        .is_started();

                    self.midi_out
                        .send(
                            &MidiMessage::NoteOn {
                                channel: if !is_started {
                                    6 // 100% brightness
                                } else {
                                    0xd // blinking 1/8
                                },
                                note_number: self
                                    .get_button_note_number(*button_id)
                                    .ok_or(DemexInputDeviceError::ButtonNotFound(*button_id))?,
                                key_velocity: 3, // white,
                            }
                            .to_bytes(),
                        )
                        .map_err(|err| DemexInputDeviceError::MidirError(err.into()))?;
                }
                DemexInputButton::SelectivePreset { .. } => {
                    self.midi_out
                        .send(
                            &MidiMessage::NoteOn {
                                channel: 6, // 100% white
                                note_number: self
                                    .get_button_note_number(*button_id)
                                    .ok_or(DemexInputDeviceError::ButtonNotFound(*button_id))?,
                                key_velocity: 9, // orange,
                            }
                            .to_bytes(),
                        )
                        .map_err(|err| DemexInputDeviceError::MidirError(err.into()))?;
                }
                DemexInputButton::Macro { .. } => {
                    self.midi_out
                        .send(
                            &MidiMessage::NoteOn {
                                channel: 0, // turn on
                                note_number: self
                                    .get_button_note_number(*button_id)
                                    .ok_or(DemexInputDeviceError::ButtonNotFound(*button_id))?,
                                key_velocity: 1, // on
                            }
                            .to_bytes(),
                        )
                        .map_err(|err| DemexInputDeviceError::MidirError(err.into()))?;
                }
                DemexInputButton::FixtureSelector { fixture_selector } => {
                    self.midi_out
                        .send(
                            &MidiMessage::NoteOn {
                                channel: if global_fixture_selector.as_ref().is_some_and(
                                    |global_fixtuer_selector| {
                                        global_fixtuer_selector == fixture_selector
                                    },
                                ) {
                                    10 // pulsing 1/2
                                } else {
                                    6 // 100%
                                },
                                note_number: self
                                    .get_button_note_number(*button_id)
                                    .ok_or(DemexInputDeviceError::ButtonNotFound(*button_id))?,
                                key_velocity: 106, // orange,
                            }
                            .to_bytes(),
                        )
                        .map_err(|err| DemexInputDeviceError::MidirError(err.into()))?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn poll(
        &self,
    ) -> Result<
        Vec<crate::input::message::DemexInputDeviceMessage>,
        crate::input::error::DemexInputDeviceError,
    > {
        let values = self
            .rx
            .try_iter()
            .flat_map(|midi_msg| match midi_msg {
                MidiMessage::NoteOn {
                    channel,
                    note_number,
                    ..
                } => self
                    .get_button_idx(channel, note_number)
                    .map(DemexInputDeviceMessage::ButtonPressed),
                MidiMessage::NoteOff {
                    channel,
                    note_number,
                    ..
                } => self
                    .get_button_idx(channel, note_number)
                    .map(DemexInputDeviceMessage::ButtonReleased),
                MidiMessage::ControlChange {
                    channel,
                    control_code,
                    control_value,
                } => self.get_fader_idx(channel, control_code).map(|idx| {
                    DemexInputDeviceMessage::FaderValueChanged(idx, control_value as f32 / 127.0)
                }),
            })
            .collect::<Vec<_>>();

        Ok(values)
    }

    fn profile_type(&self) -> super::DemexInputDeviceProfileType {
        DemexInputDeviceProfileType::ApcMiniMk2
    }
}
