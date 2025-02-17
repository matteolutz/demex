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
}

impl std::fmt::Debug for ApcMiniMk2InputDeviceProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApcMiniMk2InputDeviceProfile")
    }
}

impl ApcMiniMk2InputDeviceProfile {
    pub fn new() -> Result<Self, DemexInputDeviceError> {
        let (tx, rx) = mpsc::channel();

        thread::spawn(|| {
            let midi_in = midir::MidiInput::new("demex-midi-input").unwrap();

            let ports = midi_in.ports();
            let port = ports.first().unwrap();

            let _conn_in = midi_in.connect(
                port,
                APC_MINI_MK_2_NAME,
                move |_, msg, _| {
                    if let Some(midi_msg) = MidiMessage::from_bytes(msg) {
                        tx.send(midi_msg).unwrap();
                    }
                },
                (),
            );

            loop {
                thread::sleep(Duration::from_millis(16));
            }
        });

        Ok(Self { rx })
    }
}

impl DemexInputDeviceProfile for ApcMiniMk2InputDeviceProfile {
    fn poll(
        &self,
    ) -> Result<
        Vec<crate::input::message::DemexInputDeviceMessage>,
        crate::input::error::DemexInputDeviceError,
    > {
        let values = self
            .rx
            .try_iter()
            .map(|midi_msg| match midi_msg {
                MidiMessage::NoteOn { .. } => Some(DemexInputDeviceMessage::ButtonPressed(0)),
                MidiMessage::NoteOff { .. } => Some(DemexInputDeviceMessage::ButtonReleased(0)),
                MidiMessage::ControlChange {
                    channel: _,
                    control_code,
                    control_value,
                } => {
                    if control_code == 19 {
                        Some(DemexInputDeviceMessage::FaderValueChanged(
                            0,
                            control_value as f32 / 127.0,
                        ))
                    } else {
                        None
                    }
                }
            })
            .flatten()
            .collect::<Vec<_>>();

        Ok(values)
    }

    fn profile_type(&self) -> super::DemexInputDeviceProfileType {
        DemexInputDeviceProfileType::ApcMiniMk2
    }
}
