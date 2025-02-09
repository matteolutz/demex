use std::{sync::mpsc, thread};

use open_dmx::DMXSerial;

use crate::dmx::DMXOutput;

type DmxData = [u8; 512];

#[derive(Debug)]
pub struct DMXSerialOutput {
    tx: mpsc::Sender<DmxData>,
    universe: u16,
}

unsafe impl Sync for DMXSerialOutput {}

impl DMXSerialOutput {
    pub fn new(serial_port: String, universe: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let mut serial = DMXSerial::open_sync(serial_port.as_str()).unwrap();

            loop {
                let data = rx.recv().unwrap();
                serial.set_channels(data);
                serial.update().unwrap();
            }
        });

        Ok(Self { tx, universe })
    }
}

impl DMXOutput for DMXSerialOutput {
    fn send(&mut self, universe: u16, data: &[u8; 512]) -> Result<(), Box<dyn std::error::Error>> {
        if universe != self.universe {
            return Ok(());
        }

        self.tx.send(*data)?;
        Ok(())
    }
}
