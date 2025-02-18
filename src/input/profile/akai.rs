use std::{sync::mpsc, thread, time::Duration};

use crate::input::{
    error::DemexInputDeviceError, message::DemexInputDeviceMessage, midi::MidiMessage,
    DemexInputDeviceProfile,
};

use super::DemexInputDeviceProfileType;

// **Ressources**
// https://cdn.inmusicbrands.com/akai/attachments/APC%20mini%20mk2%20-%20Communication%20Protocol%20-%20v1.0.pdf

const APC_MINI_MK_2_NAME: &str = "Akai APC Mini MK2";

pub struct ApcMiniMk2InputDeviceProfile {
    rx: mpsc::Receiver<MidiMessage>,
    #[allow(dead_code)]
    midi_out: midir::MidiOutputConnection,
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

        thread::spawn(|| {
            let midi_in = midir::MidiInput::new("demex-midi-input")
                .map_err(|err| DemexInputDeviceError::MidirError(err.into()))
                .unwrap();

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

            let _conn_in = midi_in
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
                .map_err(|err| DemexInputDeviceError::MidirError(err.into()))
                .unwrap();

            loop {
                thread::sleep(Duration::from_millis(16));
            }
        });

        Ok(Self {
            rx,
            midi_out: connection,
        })
    }

    pub fn get_button_note_number(&self, idx: u32) -> Option<u8> {
        match idx {
            0..=63 => Some(idx as u8),
            100..=107 => Some(idx as u8 + 0x64),
            200..=207 => Some(idx as u8 + 0x70),
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
        _device_config: &crate::input::device::DemexInputDeviceConfig,
        _updatable_handler: &crate::fixture::updatables::UpdatableHandler,
    ) -> Result<(), DemexInputDeviceError> {
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
