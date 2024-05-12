use open_dmx::DMXSerial;

use crate::dmx::DMXOutput;

#[derive(Debug)]
pub struct DMXSerialOutput {
    serial: DMXSerial,
}

impl DMXSerialOutput {
    pub fn new(serial_port: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            serial: DMXSerial::open_sync(serial_port)?,
        })
    }
}

impl DMXOutput for DMXSerialOutput {
    fn send(&mut self, _universe: u16, data: &[u8; 512]) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: multiple universes with multiple serial ports

        self.serial.set_channels(*data);
        self.serial.update()?;

        Ok(())
    }
}
