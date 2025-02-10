use std::{
    sync::mpsc::{self, TryRecvError},
    thread,
};

use open_dmx::DMXSerial;

use super::DmxData;

pub fn start_serial_output_thread(rx: mpsc::Receiver<DmxData>, serial_port: String, universe: u16) {
    thread::spawn(move || {
        let mut serial = DMXSerial::open_sync(serial_port.as_str()).unwrap();

        loop {
            let recv_result = rx.try_recv();

            if let Ok((send_universe, send_universe_data)) = recv_result {
                if universe != send_universe {
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
