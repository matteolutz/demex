use crate::dmx::DMXOutput;

#[derive(Debug)]
pub enum DebugDummyOutputVerbosity {
    Verbose,
    Quiet,
    Silent,
}

#[derive(Debug)]
pub struct DebugDummyOutput {
    verbosity: DebugDummyOutputVerbosity,
}

impl DebugDummyOutput {
    pub fn new(verbosity: DebugDummyOutputVerbosity) -> Self {
        Self { verbosity }
    }
}

impl DMXOutput for DebugDummyOutput {
    fn send(&mut self, universe: u16, data: &[u8; 512]) -> Result<(), Box<dyn std::error::Error>> {
        match self.verbosity {
            DebugDummyOutputVerbosity::Verbose => {
                println!("Sending data on universe {}:\n{:?}", universe, data);
            }
            DebugDummyOutputVerbosity::Quiet => {
                println!("Sending data on universe {}", universe);
            }
            DebugDummyOutputVerbosity::Silent => {}
        }

        Ok(())
    }
}
