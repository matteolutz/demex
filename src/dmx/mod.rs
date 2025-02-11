use std::{
    fmt,
    sync::mpsc::{self},
};

use artnet::{start_artnet_output_thread, start_broadcast_artnet_output_thread};
use debug::{start_debug_output_thread, DebugOutputVerbosity};
use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};
use serial::start_serial_output_thread;

pub mod artnet;
pub mod debug;
pub mod serial;

pub trait DemexDmxOutputTrait: fmt::Debug {
    fn send(&self, universe: u16, data: &[u8; 512]) -> Result<(), Box<dyn std::error::Error>>;
}

pub type DmxData = (u16, [u8; 512]);

#[derive(Debug, Clone, EguiProbe, Serialize, Deserialize)]
pub enum DemexDmxOutputConfig {
    Debug(DebugOutputVerbosity),

    Serial {
        serial_port: String,
        universe: u16,
    },

    Artnet {
        broadcast: bool,
        bind_ip: Option<String>,
    },
}

impl DemexDmxOutputConfig {
    pub fn num_threads(&self) -> usize {
        match self {
            Self::Debug(_) => 1,
            Self::Serial { .. } => 1,
            Self::Artnet { .. } => 2,
        }
    }
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
            Self::Artnet { broadcast, bind_ip } => {
                if *broadcast {
                    start_broadcast_artnet_output_thread(rx, bind_ip.clone())
                } else {
                    start_artnet_output_thread(rx, bind_ip.clone())
                }
            }
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
    fn send(&self, universe: u16, data: &[u8; 512]) -> Result<(), Box<dyn std::error::Error>> {
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
