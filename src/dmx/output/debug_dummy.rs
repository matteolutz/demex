use crate::dmx::DMXOutput;

#[derive(Debug)]
pub struct DebugDummyOutput {
    verbose: bool,
}

impl DebugDummyOutput {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}

impl DMXOutput for DebugDummyOutput {
    fn send(&mut self, universe: u16, data: &[u8; 512]) -> Result<(), Box<dyn std::error::Error>> {
        if self.verbose {
            println!("Sending data on universe {}:\n{:?}", universe, data);
        } else {
            println!("Sending data on universe {}", universe);
        }

        Ok(())
    }
}
