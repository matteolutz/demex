use std::sync::mpsc;

use serde::{Deserialize, Serialize};

use crate::input::{
    error::DemexInputDeviceError,
    midi::{MidiMessage, device_mode::MidiInOutDeviceMode, error::MidiError},
};

lazy_static::lazy_static! {
    static ref DEMEX_MIDI_RETRIES: usize = std::env::var("DEMEX_MIDI_RETRIES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MidiInOutIdentifier {
    in_id: Option<String>,
    out_id: Option<String>,
}

pub struct MidiInOutDevice {
    name: String,

    rx: mpsc::Receiver<MidiMessage>,

    in_conn: Option<(midir::MidiInputPort, midir::MidiInputConnection<()>)>,
    out_conn: Option<(midir::MidiOutputPort, midir::MidiOutputConnection)>,
}

impl MidiInOutDevice {
    pub fn new<NameFilter>(
        name: String,
        name_filter: NameFilter,
        mode: MidiInOutDeviceMode,
        id: Option<MidiInOutIdentifier>,
    ) -> Self
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
            id,
        )
    }

    pub fn with_filters<OutputFilter, InputFilter>(
        name: String,
        output_filter: OutputFilter,
        input_filter: InputFilter,
        id: Option<MidiInOutIdentifier>,
    ) -> Self
    where
        OutputFilter: Fn(&midir::MidiOutput, &midir::MidiOutputPort) -> bool,
        InputFilter: Fn(&midir::MidiInput, &midir::MidiInputPort) -> bool,
    {
        let (tx, rx) = mpsc::channel();

        log::debug!("Connecting to MIDI device \"{}\"", name);

        let out_conn = Self::get_out_connection(
            &name,
            output_filter,
            id.as_ref().and_then(|id| id.out_id.as_ref()),
        )
        .inspect_err(|err| {
            log::error!(
                "Failed to establish output connection to \"{}\": {}",
                name,
                err
            )
        })
        .ok()
        .flatten()
        .inspect(|(port, _)| {
            log::debug!("Connected to \"{}\" output (Port ID: {})", name, port.id())
        });

        let in_conn = Self::get_in_connection(
            &name,
            input_filter,
            tx,
            id.as_ref().and_then(|id| id.in_id.as_ref()),
        )
        .inspect_err(|err| {
            log::error!(
                "Failed to establish input connection to \"{}\": {}",
                name,
                err
            )
        })
        .ok()
        .flatten()
        .inspect(|(port, _)| {
            log::debug!("Connected to \"{}\" input (Port ID: {})", name, port.id())
        });

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
        id: Option<&String>,
    ) -> Result<Option<(midir::MidiOutputPort, midir::MidiOutputConnection)>, MidiError>
    where
        Filter: Fn(&midir::MidiOutput, &midir::MidiOutputPort) -> bool,
    {
        let midi_out = midir::MidiOutput::new(format!("demex-midi-output-{}", name).as_str())
            .map_err(|err| MidiError::MidirError(err.into()))?;

        for i in 0..=*DEMEX_MIDI_RETRIES {
            log::debug!("Looking for output port (i = {})", i);

            let out_ports = midi_out.ports();
            for port in &out_ports {
                log::debug!(
                    "Found output port: {:?} (id = {:?})",
                    midi_out.port_name(port),
                    port.id()
                );
            }

            let out_port = out_ports
                .into_iter()
                .inspect(|port| log::debug!("Checking output port {:?}", midi_out.port_name(port)))
                .find(|port| filter(&midi_out, port) && id.is_none_or(|id| &port.id() == id));

            if let Some(port) = out_port {
                log::debug!(
                    "Using output port {:?} with id {:?}",
                    midi_out.port_name(&port),
                    port.id()
                );

                let connection = midi_out
                    .connect(&port, format!("demex-midi-output-port-{}", name).as_str())
                    .map_err(|err| MidiError::MidirError(err.into()))?;

                return Ok(Some((port, connection)));
            } else {
                log::debug!("Found no output port");
            }
        }

        Ok(None)
    }

    fn get_in_connection<Filter>(
        name: &str,
        filter: Filter,
        tx: mpsc::Sender<MidiMessage>,
        id: Option<&String>,
    ) -> Result<Option<(midir::MidiInputPort, midir::MidiInputConnection<()>)>, MidiError>
    where
        Filter: Fn(&midir::MidiInput, &midir::MidiInputPort) -> bool,
    {
        let midi_in = midir::MidiInput::new(format!("demex-midi-input-{}", name).as_str())
            .map_err(|err| MidiError::MidirError(err.into()))?;

        for i in 0..=*DEMEX_MIDI_RETRIES {
            log::debug!("Looking for input port (i = {})", i);

            let in_ports = midi_in.ports();
            for port in &in_ports {
                log::debug!(
                    "Found input port: {:?} (id = {:?})",
                    midi_in.port_name(port),
                    port.id()
                );
            }

            let in_port = in_ports
                .into_iter()
                .inspect(|port| log::debug!("Checking input port {:?}", midi_in.port_name(port)))
                .find(|port| filter(&midi_in, port) && id.is_none_or(|id| &port.id() == id));

            if let Some(port) = in_port {
                log::debug!(
                    "Using input port {:?} with id {:?}",
                    midi_in.port_name(&port),
                    port.id()
                );

                let connection = midi_in
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
                    .map_err(|err| MidiError::MidirError(err.into()))?;

                return Ok(Some((port, connection)));
            } else {
                log::debug!("Found no input port");
            }
        }

        Ok(None)
    }
}

impl MidiInOutDevice {
    pub fn has_input(&self) -> bool {
        self.in_conn.is_some()
    }

    pub fn has_output(&self) -> bool {
        self.out_conn.is_some()
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn output(&self) -> Option<&midir::MidiOutputConnection> {
        self.out_conn.as_ref().map(|(_, conn)| conn)
    }

    pub fn output_mut(&mut self) -> Option<&mut midir::MidiOutputConnection> {
        self.out_conn.as_mut().map(|(_, conn)| conn)
    }

    pub fn send(&mut self, message: MidiMessage) -> Result<(), DemexInputDeviceError> {
        let Some(output) = self.output_mut() else {
            return Err(DemexInputDeviceError::OperationNotSupported);
        };

        output
            .send(&message.to_bytes())
            .map_err(|err| DemexInputDeviceError::MidirError(err.into()))
    }

    pub fn input(&self) -> Option<&midir::MidiInputConnection<()>> {
        self.in_conn.as_ref().map(|(_, conn)| conn)
    }

    pub fn input_rx(&self) -> &mpsc::Receiver<MidiMessage> {
        &self.rx
    }
}
