use std::sync::mpsc;

use crate::input::{
    error::DemexInputDeviceError, message::DemexInputDeviceMessage, midi::MidiMessage,
    timecode::packet::TimecodePacket, DemexInputDeviceProfile,
};

pub struct GenericMidiProfile {
    rx: mpsc::Receiver<MidiMessage>,
    midi_in: Option<midir::MidiInputConnection<()>>,
}

impl std::fmt::Debug for GenericMidiProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GenericMidiProfile")
    }
}

impl GenericMidiProfile {
    fn get_conn_in(
        tx: mpsc::Sender<MidiMessage>,
    ) -> Result<midir::MidiInputConnection<()>, DemexInputDeviceError> {
        let midi_in = midir::MidiInput::new("demex-midi-input")
            .map_err(|err| DemexInputDeviceError::MidirError(err.into()))?;

        let in_ports = midi_in.ports();
        let in_port = in_ports
            .first()
            .ok_or(DemexInputDeviceError::InputDeviceNotFound(
                "GenericMidiDevice".to_owned(),
            ))?;

        midi_in
            .connect(
                in_port,
                "GenericMidiDevice",
                move |_, msg, _| {
                    if let Some(midi_msg) = MidiMessage::from_bytes(msg) {
                        tx.send(midi_msg).unwrap();
                    } else {
                        log::warn!("failed to deserialize midi bytes: {:02X?}", msg);
                    }
                },
                (),
            )
            .map_err(|err| DemexInputDeviceError::MidirError(err.into()))
    }

    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        let conn_in = Self::get_conn_in(tx);

        Self {
            rx,
            midi_in: conn_in.ok(),
        }
    }
}

impl DemexInputDeviceProfile for GenericMidiProfile {
    fn update_out(
        &mut self,
        _: &crate::input::device::DemexInputDeviceConfig,
        _: &crate::fixture::presets::PresetHandler,
        _: &crate::fixture::updatables::UpdatableHandler,
        _: &crate::fixture::timing::TimingHandler,
        _: &Option<crate::fixture::selection::FixtureSelection>,
    ) -> Result<(), crate::input::error::DemexInputDeviceError> {
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
                MidiMessage::Timecode {
                    rate,
                    hour,
                    minute,
                    second,
                    frame,
                } => Some(DemexInputDeviceMessage::Timecode(TimecodePacket {
                    hour,
                    minute,
                    second,
                    frame,
                    rate: rate.into(),
                })),
                MidiMessage::TimecodeQuarterFrame { piece } => {
                    Some(DemexInputDeviceMessage::TimecodeQuarterFrame { piece })
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        Ok(values)
    }

    fn profile_type(&self) -> super::DemexInputDeviceProfileType {
        super::DemexInputDeviceProfileType::GenericMidi
    }

    fn is_enabled(&self) -> bool {
        self.midi_in.is_some()
    }
}
