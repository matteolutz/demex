use std::sync::mpsc;

use crate::input::midi::{device_mode::MidiInOutDeviceMode, error::MidiError, MidiMessage};

pub struct MidiInOutDevice {
    name: String,

    rx: mpsc::Receiver<MidiMessage>,

    in_conn: Option<midir::MidiInputConnection<()>>,
    out_conn: Option<midir::MidiOutputConnection>,
}

impl MidiInOutDevice {
    pub fn new<NameFilter>(name: String, name_filter: NameFilter, mode: MidiInOutDeviceMode) -> Self
    where
        NameFilter: Fn(&str) -> bool,
    {
        Self::with_filters(
            name,
            |midi_out: &midir::MidiOutput, port| {
                mode.is_output()
                    && midi_out
                        .port_name(port)
                        .is_ok_and(|name| name_filter(&name))
            },
            |midi_in: &midir::MidiInput, port| {
                mode.is_input() && midi_in.port_name(port).is_ok_and(|name| name_filter(&name))
            },
        )
    }

    pub fn with_filters<OutputFilter, InputFilter>(
        name: String,
        output_filter: OutputFilter,
        input_filter: InputFilter,
    ) -> Self
    where
        OutputFilter: Fn(&midir::MidiOutput, &midir::MidiOutputPort) -> bool,
        InputFilter: Fn(&midir::MidiInput, &midir::MidiInputPort) -> bool,
    {
        let (tx, rx) = mpsc::channel();

        let out_conn = Self::get_out_connection(&name, output_filter)
            .inspect_err(|err| {
                log::error!(
                    "Failed to establish output connection to \"{}\": {}",
                    name,
                    err
                )
            })
            .ok()
            .flatten();

        let in_conn = Self::get_in_connection(&name, input_filter, tx)
            .inspect_err(|err| {
                log::error!(
                    "Failed to establish input connection to \"{}\": {}",
                    name,
                    err
                )
            })
            .ok()
            .flatten();

        MidiInOutDevice {
            name,
            rx,
            in_conn,
            out_conn,
        }
    }
}

impl MidiInOutDevice {
    fn get_out_connection<Filter>(
        name: &str,
        filter: Filter,
    ) -> Result<Option<midir::MidiOutputConnection>, MidiError>
    where
        Filter: Fn(&midir::MidiOutput, &midir::MidiOutputPort) -> bool,
    {
        let midi_out = midir::MidiOutput::new(format!("demex-midi-output-{}", name).as_str())
            .map_err(|err| MidiError::MidirError(err.into()))?;

        let out_ports = midi_out.ports();
        let out_port = out_ports.into_iter().find(|port| filter(&midi_out, port));

        if let Some(port) = out_port {
            Ok(Some(
                midi_out
                    .connect(&port, format!("demex-midi-output-port-{}", name).as_str())
                    .map_err(|err| MidiError::MidirError(err.into()))?,
            ))
        } else {
            Ok(None)
        }
    }

    fn get_in_connection<Filter>(
        name: &str,
        filter: Filter,
        tx: mpsc::Sender<MidiMessage>,
    ) -> Result<Option<midir::MidiInputConnection<()>>, MidiError>
    where
        Filter: Fn(&midir::MidiInput, &midir::MidiInputPort) -> bool,
    {
        let midi_in = midir::MidiInput::new(format!("demex-midi-input-{}", name).as_str())
            .map_err(|err| MidiError::MidirError(err.into()))?;

        let in_ports = midi_in.ports();
        let in_port = in_ports.into_iter().find(|port| filter(&midi_in, port));

        if let Some(port) = in_port {
            Ok(Some(
                midi_in
                    .connect(
                        &port,
                        format!("demex-midi-input-port-{}", name).as_str(),
                        move |_, msg, _| {
                            if let Some(midi_msg) = MidiMessage::from_bytes(msg) {
                                let _ = tx.send(midi_msg).inspect_err(|err| {
                                    log::warn!("Failed to send MIDI message: {}", err)
                                });
                            } else {
                                log::debug!("Failed to deserialize MIDI bytes {:02X?}", msg);
                            }
                        },
                        (),
                    )
                    .map_err(|err| MidiError::MidirError(err.into()))?,
            ))
        } else {
            Ok(None)
        }
    }
}

impl MidiInOutDevice {
    pub fn has_input(&self) -> bool {
        self.in_conn.is_some()
    }

    pub fn has_output(&self) -> bool {
        self.out_conn.is_some()
    }

    pub fn input(&self) -> Option<&midir::MidiInputConnection<()>> {
        self.in_conn.as_ref()
    }

    pub fn output(&self) -> Option<&midir::MidiOutputConnection> {
        self.out_conn.as_ref()
    }
}
