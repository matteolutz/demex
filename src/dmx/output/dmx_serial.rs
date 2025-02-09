use std::{sync::mpsc, thread};

use open_dmx::DMXSerial;

use crate::dmx::DMXOutput;

type DmxData = [u8; 512];

#[derive(Debug)]
pub struct DMXSerialOutput {
    // serial: DMXSerial,
    tx: mpsc::Sender<DmxData>,
}

unsafe impl Sync for DMXSerialOutput {}

impl DMXSerialOutput {
    pub fn new(serial_port: String) -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let mut serial = DMXSerial::open_sync(serial_port.as_str()).unwrap();

            loop {
                let data = rx.recv().unwrap();
                serial.set_channels(data);
                serial.update().unwrap();
            }
        });

        Ok(Self {
            // serial: DMXSerial::open_sync(serial_port)?,
            tx,
        })
    }
}

impl DMXOutput for DMXSerialOutput {
    fn send(&mut self, _universe: u16, data: &[u8; 512]) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: multiple universes with multiple serial ports

        // self.serial.set_channels(*data);
        // self.serial.update()?;
        self.tx.send(data.clone())?;

        Ok(())
    }
}
