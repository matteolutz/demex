use std::{
    fmt,
    sync::mpsc::{self},
};

use artnet::start_artnet_output_thread;
use debug::{start_debug_output_thread, DebugOutputVerbosity};
use egui_probe::EguiProbe;
use serial::start_serial_output_thread;

pub mod artnet;
pub mod debug;
pub mod serial;

pub trait DemexDmxOutputTrait: fmt::Debug {
    fn send(&mut self, universe: u16, data: &[u8; 512]) -> Result<(), Box<dyn std::error::Error>>;
}

pub type DmxData = (u16, [u8; 512]);

#[derive(Debug, EguiProbe)]
pub enum DemexDmxOutputConfig {
    Debug(DebugOutputVerbosity),

    Serial { serial_port: String, universe: u16 },

    Artnet(String),
}

impl Default for DemexDmxOutputConfig {
    fn default() -> Self {
        Self::Debug(DebugOutputVerbosity::default())
    }
}

impl DemexDmxOutputConfig {
    fn spawn_thread(&self, rx: mpsc::Receiver<DmxData>) {
        match self {
            Self::Debug(output_verbosity) => start_debug_output_thread(rx, *output_verbosity),
            Self::Serial {
                serial_port,
                universe,
            } => start_serial_output_thread(rx, serial_port.clone(), *universe),
            Self::Artnet(socket_addr) => start_artnet_output_thread(rx, socket_addr.clone()),
        }
    }
}

#[derive(Debug, EguiProbe)]
pub struct DemexDmxOutput {
    config: DemexDmxOutputConfig,

    #[egui_probe(skip)]
    tx: mpsc::Sender<DmxData>,
}

impl Default for DemexDmxOutput {
    fn default() -> Self {
        DemexDmxOutputConfig::default().into()
    }
}

impl From<DemexDmxOutputConfig> for DemexDmxOutput {
    fn from(config: DemexDmxOutputConfig) -> Self {
        let (tx, rx) = mpsc::channel();

        config.spawn_thread(rx);

        Self { config, tx }
    }
}

impl DemexDmxOutputTrait for DemexDmxOutput {
    fn send(&mut self, universe: u16, data: &[u8; 512]) -> Result<(), Box<dyn std::error::Error>> {
        self.tx.send((universe, *data))?;
        Ok(())
    }
}

impl DemexDmxOutput {
    pub fn config(&self) -> &DemexDmxOutputConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut DemexDmxOutputConfig {
        &mut self.config
    }
}
