use std::{
    fmt,
    sync::mpsc::{self},
};

use artnet::{
    start_artnet_output_thread, start_broadcast_artnet_output_thread, ArtnetOutputConfig,
};
use debug::DebugOutputVerbosity;
use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};
use serial::SerialOutputConfig;

pub mod artnet;
pub mod debug;
pub mod serial;

pub trait DemexDmxOutputTrait: fmt::Debug {
    fn send(&mut self, universe: u16, data: &[u8; 512]) -> Result<(), Box<dyn std::error::Error>>;
}

pub type DmxData = (u16, [u8; 512]);

#[derive(Debug, Clone, EguiProbe, Serialize, Deserialize)]
pub enum DemexDmxOutputConfig {
    Debug(DebugOutputVerbosity),
    Serial(SerialOutputConfig),
    Artnet(ArtnetOutputConfig),
}

impl DemexDmxOutputConfig {
    pub fn num_threads(&self) -> usize {
        match self {
            Self::Debug(_) => 0,
            Self::Serial(_) => 1,
            Self::Artnet(_) => 1,
        }
    }
}

impl Default for DemexDmxOutputConfig {
    fn default() -> Self {
        Self::Debug(DebugOutputVerbosity::default())
    }
}

#[derive(Debug)]
pub enum DemexDmxOutputData {
    Artnet {
        tx: mpsc::Sender<DmxData>,
        config: ArtnetOutputConfig,
    },
    Serial {
        tx: mpsc::Sender<DmxData>,
        config: SerialOutputConfig,
    },
    Debug(DebugOutputVerbosity),
}

impl DemexDmxOutputTrait for DemexDmxOutputData {
    fn send(&mut self, universe: u16, data: &[u8; 512]) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::Artnet { tx, .. } | Self::Serial { tx, .. } => tx.send((universe, *data))?,
            Self::Debug(_) => (),
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct DemexDmxOutput {
    data: DemexDmxOutputData,
    config: DemexDmxOutputConfig,
}

impl Default for DemexDmxOutput {
    fn default() -> Self {
        DemexDmxOutputConfig::default().into()
    }
}

impl From<DemexDmxOutputConfig> for DemexDmxOutput {
    fn from(config: DemexDmxOutputConfig) -> Self {
        let data = match &config {
            DemexDmxOutputConfig::Artnet(config) => {
                let (tx, rx) = mpsc::channel();
                if config.broadcast {
                    start_broadcast_artnet_output_thread(rx, config.clone());
                } else {
                    start_artnet_output_thread(rx, config.clone());
                }

                DemexDmxOutputData::Artnet {
                    tx,
                    config: config.clone(),
                }
            }
            DemexDmxOutputConfig::Debug(verbosity) => DemexDmxOutputData::Debug(*verbosity),
            DemexDmxOutputConfig::Serial(config) => {
                let (tx, rx) = mpsc::channel();
                serial::start_serial_output_thread(rx, config.clone());

                DemexDmxOutputData::Serial {
                    tx,
                    config: config.clone(),
                }
            }
        };

        Self { data, config }
    }
}

impl DemexDmxOutputTrait for DemexDmxOutput {
    fn send(&mut self, universe: u16, data: &[u8; 512]) -> Result<(), Box<dyn std::error::Error>> {
        if let Err(err) = self.data.send(universe, data) {
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
