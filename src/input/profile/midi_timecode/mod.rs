use std::sync::mpsc;

use crate::input::{
    error::DemexInputDeviceError, message::DemexInputDeviceMessage, midi::MidiMessage,
    timecode::packet::TimecodePacket, DemexInputDeviceProfile,
};

pub struct MidiTimecodeProfile {
    #[allow(dead_code)]
    midi_in_device: String,

    rx: mpsc::Receiver<MidiMessage>,
    midi_in: Option<midir::MidiInputConnection<()>>,
}

impl std::fmt::Debug for MidiTimecodeProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GenericMidiProfile")
    }
}

impl MidiTimecodeProfile {
    fn get_conn_in(
        midi_in_device: &str,
        tx: mpsc::Sender<MidiMessage>,
    ) -> Result<midir::MidiInputConnection<()>, DemexInputDeviceError> {
        let midi_in = midir::MidiInput::new("demex-midi-input")
            .map_err(|err| DemexInputDeviceError::MidirError(err.into()))?;

        let in_ports = midi_in.ports();

        for in_port in &in_ports {
            log::debug!("Found MIDI in port: {:?}", midi_in.port_name(in_port));
        }

        let in_port = in_ports
            .iter()
            .inspect(|p| log::debug!("Found MIDI in port: {:?}", midi_in.port_name(p)))
            .find(|p| {
                midi_in
                    .port_name(p)
                    .is_ok_and(|port_name| port_name == midi_in_device)
            })
            .ok_or(DemexInputDeviceError::InputDeviceNotFound(
                midi_in_device.to_owned(),
            ))?;

        midi_in
            .connect(
                in_port,
                midi_in_device,
                move |_, msg, _| {
                    if let Some(midi_msg) = MidiMessage::from_bytes(msg) {
                        tx.send(midi_msg).unwrap();
                    } else {
                        log::debug!("failed to deserialize midi bytes: {:02X?}", msg);
                    }
                },
                (),
            )
            .map_err(|err| DemexInputDeviceError::MidirError(err.into()))
    }

    pub fn new(midi_in_device: String) -> Self {
        let (tx, rx) = mpsc::channel();

        let conn_in = Self::get_conn_in(&midi_in_device, tx);

        Self {
            midi_in_device,
            rx,
            midi_in: conn_in.ok(),
        }
    }
}

impl DemexInputDeviceProfile for MidiTimecodeProfile {
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

    fn is_enabled(&self) -> bool {
        self.midi_in.is_some()
    }
}
