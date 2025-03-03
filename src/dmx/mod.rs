use std::{
    fmt,
    sync::mpsc::{self},
};

use artnet::{start_artnet_output_thread, start_broadcast_artnet_output_thread};
use debug::{start_debug_output_thread, DebugOutputVerbosity};
use egui::menu::MenuResponse;
use egui_probe::EguiProbe;
use open_dmx::DMXSerial;
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
        #[serde(default)]
        broadcast_addresses: Vec<String>,
        bind_ip: Option<String>,
    },
}

impl DemexDmxOutputConfig {
    pub fn num_threads(&self) -> usize {
        match self {
            Self::Debug(_) => 1,
            Self::Serial { .. } => 1,
            Self::Artnet { .. } => 1,
        }
    }
}

impl Default for DemexDmxOutputConfig {
    fn default() -> Self {
        Self::Debug(DebugOutputVerbosity::default())
    }
}

impl DemexDmxOutputConfig {
    fn spawn_thread(
        &self,
        tx: mpsc::Sender<DmxData>,
        rx: mpsc::Receiver<DmxData>,
    ) -> Option<mpsc::Sender<DmxData>> {
        match self {
            Self::Debug(output_verbosity) => {
                start_debug_output_thread(rx, *output_verbosity);
                None
            }
            Self::Serial {
                serial_port,
                universe,
            } => {
                start_serial_output_thread(rx, serial_port.clone(), *universe);
                None
            }
            Self::Artnet {
                broadcast,
                broadcast_addresses,
                bind_ip,
            } => {
                if *broadcast {
                    start_broadcast_artnet_output_thread(rx, bind_ip.clone())
                } else {
                    start_artnet_output_thread(rx, bind_ip.clone(), broadcast_addresses.clone())
                }

                Some(tx)
            }
        }
    }

    pub fn update_sync(&self, data: &[u8; 512]) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::Debug(_) => Ok(()),
            Self::Serial { .. } => Ok(()),
            Self::Artnet { .. } => Ok(()),
        }
    }
}

pub enum DemexDmxSpecificOutput {
    Artnet { tx: mpsc::Sender<DmxData> },
    Serial { serial_port: DMXSerial },
    Debug,
}

#[derive(Debug, EguiProbe)]
pub struct DemexDmxOutput {
    config: DemexDmxOutputConfig,

    #[egui_probe(skip)]
    tx: Option<mpsc::Sender<DmxData>>,
}

impl Default for DemexDmxOutput {
    fn default() -> Self {
        DemexDmxOutputConfig::default().into()
    }
}

impl From<DemexDmxOutputConfig> for DemexDmxOutput {
    fn from(config: DemexDmxOutputConfig) -> Self {
        let (tx, rx) = mpsc::channel();

        let tx = config.spawn_thread(tx, rx);

        Self { config, tx }
    }
}

impl DemexDmxOutputTrait for DemexDmxOutput {
    fn send(&self, universe: u16, data: &[u8; 512]) -> Result<(), Box<dyn std::error::Error>> {
        // If self.tx has a value, we know, that we spawned a thread and can send the data to it.
        if let Some(tx) = &self.tx {
            tx.send((universe, *data))?;
        }

        if let Err(err) = self.config.update_sync(data) {
            log::warn!("Error updating sync for {:?}: {}", self, err);
        }

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
