use std::{
    fmt,
    sync::mpsc::{self},
};

use artnet::{
    ArtnetOutputConfig, start_artnet_output_thread, start_broadcast_artnet_output_thread,
};
use debug::DebugOutputVerbosity;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serial::UsbSerialOutputConfig;

use demex_headless::id::DemexProtoDeviceId;

pub mod artnet;
pub mod debug;
pub mod serial;

pub trait DemexDmxOutputTrait: fmt::Debug {
    fn send(&mut self, universe: u16, data: &[u8; 512]) -> Result<(), Box<dyn std::error::Error>>;
}

pub type DmxData = (u16, [u8; 512]);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DemexDmxOutputConfigData {
    Debug(DebugOutputVerbosity),

    UsbSerial(UsbSerialOutputConfig),
    Artnet(ArtnetOutputConfig),
}

impl DemexDmxOutputConfigData {
    pub fn name(&self) -> String {
        match self {
            DemexDmxOutputConfigData::Debug(_) => "Debug".into(),
            DemexDmxOutputConfigData::UsbSerial(config) => config
                .usb_port
                .0
                .product
                .clone()
                .unwrap_or_else(|| "USB Debug".into()),
            DemexDmxOutputConfigData::Artnet(_) => "ArtNet".into(),
        }
    }
}

impl std::fmt::Display for DemexDmxOutputConfigData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UsbSerial(serial_config) => write!(
                f,
                "Universe {} on {} (RTS: {})",
                serial_config.universe,
                serial_config.usb_port.to_string(),
                serial_config.enable_rts
            ),
            Self::Artnet(artnet_config) => {
                write!(
                    f,
                    "Bound to {} with universes [{}] (Broadcast: {})",
                    artnet_config
                        .bind_ip
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or("::0"),
                    artnet_config.universes.iter().join(", "),
                    artnet_config.broadcast
                )
            }
            Self::Debug(verbosity) => write!(f, "Verbosity: {:?}", verbosity),
        }
    }
}

impl Default for DemexDmxOutputConfigData {
    fn default() -> Self {
        Self::Debug(DebugOutputVerbosity::Quiet)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemexDmxOutputConfig {
    data: DemexDmxOutputConfigData,
    device_id: DemexProtoDeviceId,

    #[serde(default)]
    disabled: bool,
}

impl DemexDmxOutputConfig {
    pub fn new(data: DemexDmxOutputConfigData, device_id: DemexProtoDeviceId) -> Self {
        Self {
            data,
            device_id,
            disabled: false,
        }
    }
}

impl std::fmt::Display for DemexDmxOutputConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.data.fmt(f)
    }
}

impl DemexDmxOutputConfig {
    pub fn universes(&self) -> Option<Vec<u16>> {
        match &self.data {
            DemexDmxOutputConfigData::Debug(_) => None,
            DemexDmxOutputConfigData::Artnet(config) => Some(config.universes.clone()),
            DemexDmxOutputConfigData::UsbSerial(config) => Some(vec![config.universe]),
        }
    }

    pub fn num_threads(&self) -> usize {
        match &self.data {
            DemexDmxOutputConfigData::Debug(_) => 0,
            DemexDmxOutputConfigData::UsbSerial(_) => 1,
            DemexDmxOutputConfigData::Artnet(_) => 1,
        }
    }

    pub fn name(&self) -> String {
        self.data.name()
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

impl Default for DemexDmxOutputConfig {
    fn default() -> Self {
        Self {
            data: DemexDmxOutputConfigData::default(),
            device_id: DemexProtoDeviceId::Controller,
            disabled: false,
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
        config: UsbSerialOutputConfig,
    },
    Debug(DebugOutputVerbosity),
    None,
}

impl DemexDmxOutputTrait for DemexDmxOutputData {
    fn send(&mut self, universe: u16, data: &[u8; 512]) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::Artnet { tx, .. } | Self::Serial { tx, .. } => tx.send((universe, *data))?,
            Self::Debug(verbosity) => match verbosity {
                DebugOutputVerbosity::Verbose => {
                    println!("Universe: {}, Data: {:?}", universe, data)
                }
                DebugOutputVerbosity::Quiet => println!("Universe: {}", universe),
                _ => {}
            },
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
        if own_device_id != config.device_id || config.disabled {
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
            DemexDmxOutputConfigData::UsbSerial(config) => {
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

#[cfg(test)]
mod tests {
    use serialport::SerialPortType;

    #[test]
    fn test_serialport() {
        let ports = serialport::available_ports().expect("No serial ports found");
        for p in ports {
            match p.port_type {
                SerialPortType::UsbPort(usb) => println!("{:?}", usb),
                _ => {}
            }
        }
    }
}
