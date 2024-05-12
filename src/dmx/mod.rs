pub mod output;

pub trait DMXOutput: std::fmt::Debug {
    fn send(&mut self, universe: u16, data: &[u8; 512]) -> Result<(), Box<dyn std::error::Error>>;
}
