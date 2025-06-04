use std::{
    fmt,
    sync::mpsc::{self},
};

use artnet::{
    start_artnet_output_thread, start_broadcast_artnet_output_thread, ArtnetOutputConfig,
};
use debug::DebugOutputVerbosity;
use serde::{Deserialize, Serialize};
use serial::SerialOutputConfig;

use crate::headless::id::DemexProtoDeviceId;

pub mod artnet;
pub mod debug;
pub mod serial;

pub trait DemexDmxOutputTrait: fmt::Debug {
    fn send(&mut self, universe: u16, data: &[u8; 512]) -> Result<(), Box<dyn std::error::Error>>;
}

pub type DmxData = (u16, [u8; 512]);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub enum DemexDmxOutputConfigData {
    Debug(DebugOutputVerbosity),

    Serial(SerialOutputConfig),
    Artnet(ArtnetOutputConfig),
}

impl Default for DemexDmxOutputConfigData {
    fn default() -> Self {
        Self::Debug(DebugOutputVerbosity::Quiet)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub struct DemexDmxOutputConfig {
    data: DemexDmxOutputConfigData,
    device_id: DemexProtoDeviceId,
}

impl DemexDmxOutputConfig {
    pub fn universes(&self) -> Option<Vec<u16>> {
        match &self.data {
            DemexDmxOutputConfigData::Debug(_) => None,
            DemexDmxOutputConfigData::Artnet(config) => Some(config.universes.clone()),
            DemexDmxOutputConfigData::Serial(config) => Some(vec![config.universe]),
        }
    }

    pub fn num_threads(&self) -> usize {
        match &self.data {
            DemexDmxOutputConfigData::Debug(_) => 0,
            DemexDmxOutputConfigData::Serial(_) => 1,
            DemexDmxOutputConfigData::Artnet(_) => 1,
        }
    }
}

impl Default for DemexDmxOutputConfig {
    fn default() -> Self {
        Self {
            data: DemexDmxOutputConfigData::default(),
            device_id: DemexProtoDeviceId::Controller,
        }
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
    None,
}

impl DemexDmxOutputTrait for DemexDmxOutputData {
    fn send(&mut self, universe: u16, data: &[u8; 512]) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::Artnet { tx, .. } | Self::Serial { tx, .. } => tx.send((universe, *data))?,
            Self::Debug(_) => (),
            Self::None => (),
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct DemexDmxOutput {
    data: DemexDmxOutputData,
    config: DemexDmxOutputConfig,
}

impl DemexDmxOutput {
    pub fn from_config(config: DemexDmxOutputConfig, own_device_id: DemexProtoDeviceId) -> Self {
        if own_device_id != config.device_id {
            return Self {
                data: DemexDmxOutputData::None,
                config,
            };
        }

        let data = match &config.data {
            DemexDmxOutputConfigData::Artnet(config) => {
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
            DemexDmxOutputConfigData::Debug(verbosity) => DemexDmxOutputData::Debug(*verbosity),
            DemexDmxOutputConfigData::Serial(config) => {
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
    pub fn should_output(&self) -> bool {
        !matches!(self.data, DemexDmxOutputData::None)
    }

    pub fn config(&self) -> &DemexDmxOutputConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut DemexDmxOutputConfig {
        &mut self.config
    }
}
