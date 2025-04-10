use std::{
    sync::mpsc::{self, TryRecvError},
    thread,
};

use egui_probe::EguiProbe;
use open_dmx::DMXSerial;
use serde::{Deserialize, Serialize};

use super::DmxData;

#[derive(Debug, Clone, Default, EguiProbe, Serialize, Deserialize)]
pub struct SerialOutputConfig {
    pub serial_port: String,
    pub universe: u16,
}

pub fn start_serial_output_thread(rx: mpsc::Receiver<DmxData>, config: SerialOutputConfig) {
    thread::spawn(move || {
        let mut serial = DMXSerial::open(config.serial_port.as_str()).unwrap();
        serial.set_sync();

        loop {
            let recv_result = rx.try_recv();

            if let Ok((send_universe, send_universe_data)) = recv_result {
                if config.universe != send_universe {
                    continue;
                }

                serial.set_channels(send_universe_data);
                serial.update().unwrap();
            } else if recv_result.err().unwrap() == TryRecvError::Disconnected {
                break;
            }
        }
    });
}
