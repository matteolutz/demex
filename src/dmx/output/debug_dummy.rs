use crate::dmx::DMXOutput;

#[derive(Debug)]
pub struct DebugDummyOutput {}

impl DMXOutput for DebugDummyOutput {
    fn send(&mut self, universe: u16, data: &[u8; 512]) -> Result<(), Box<dyn std::error::Error>> {
        println!("Sending data on universe {}:\n{:?}", universe, data);
        Ok(())
    }
}
