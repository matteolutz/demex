use std::{sync::mpsc, time::Duration};

use crate::input::error::DemexInputDeviceError;

use super::{MidiMessage, CC_OP, NOTE_OFF_OP, NOTE_ON_OP};

const MAX_CHUNK_SIZE: usize = 32;

pub struct UsbMidiDevice {
    device_handle: rusb::DeviceHandle<rusb::GlobalContext>,
    in_endpoint: Option<u8>,
    out_endpoint: Option<u8>,
}

impl UsbMidiDevice {
    pub fn new(
        device_handle: rusb::DeviceHandle<rusb::GlobalContext>,
        in_endpoint: Option<u8>,
        out_endpoint: Option<u8>,
    ) -> Self {
        Self {
            device_handle,
            in_endpoint,
            out_endpoint,
        }
    }

    pub fn in_device(
        device_handle: rusb::DeviceHandle<rusb::GlobalContext>,
        in_endpoint: u8,
    ) -> Self {
        Self::new(device_handle, Some(in_endpoint), None)
    }

    pub fn out_device(
        device_handle: rusb::DeviceHandle<rusb::GlobalContext>,
        out_endpoint: u8,
    ) -> Self {
        Self::new(device_handle, None, Some(out_endpoint))
    }

    pub fn in_out_device(
        device_handle: rusb::DeviceHandle<rusb::GlobalContext>,
        in_endpoint: u8,
        out_endpoint: u8,
    ) -> Self {
        Self::new(device_handle, Some(in_endpoint), Some(out_endpoint))
    }

    pub fn listen(&self, tx: mpsc::Sender<MidiMessage>) -> Result<(), DemexInputDeviceError> {
        let endpoint = self
            .in_endpoint
            .ok_or(DemexInputDeviceError::OperationNotSupported)?;

        let mut buf = [0u8; 256];
        let mut buf_tail = 0;

        loop {
            let read_bytes = self
                .device_handle
                .read_bulk(endpoint, &mut buf[buf_tail..], Duration::from_millis(1))
                .map_err(DemexInputDeviceError::RusbError)?;

            buf_tail += read_bytes;

            let mut i = 0;
            while i < buf_tail {
                let _cable_number = (buf[i] & 0xF0) >> 4;
                let cin = buf[i] & 0x0F;

                i += 1;

                match cin {
                    NOTE_ON_OP | NOTE_OFF_OP | CC_OP => {
                        // we haven't read far enough, wait for more data
                        if i + 2 > buf_tail {
                            break;
                        }

                        let midi_message = MidiMessage::from_bytes(&buf[i..i + 3]);
                        if let Some(midi_message) = midi_message {
                            tx.send(midi_message)
                                .map_err(|_| DemexInputDeviceError::MpscSendError)?;
                        }

                        buf_tail = 0;
                        break;
                    }
                    // we don't know the mesage code (todo: SysEx)
                    _ => {
                        buf_tail = 0;
                        break;
                    }
                }
            }
        }
    }

    pub fn send(&self, message: MidiMessage) -> Result<(), DemexInputDeviceError> {
        let endpoint = self
            .out_endpoint
            .ok_or(DemexInputDeviceError::OperationNotSupported)?;

        for chunk in message.to_bytes().chunks(MAX_CHUNK_SIZE) {
            self.device_handle
                .write_bulk(endpoint, chunk, Duration::from_millis(1))
                .map_err(DemexInputDeviceError::RusbError)?;
        }

        Ok(())
    }
}
