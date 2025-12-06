use std::{
    sync::mpsc::{self, TryRecvError},
    thread,
};

use open_dmx::DMXSerial;
use serde::{Deserialize, Serialize};
use serialport::SerialPortType;

use super::DmxData;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsbPortInfo(pub serialport::UsbPortInfo);
impl UsbPortInfo {
    fn matches<'a>(&self, other: &UsbPortInfo) -> bool {
        self.0.vid == other.0.vid && self.0.pid == other.0.pid
    }

    pub fn id(&self) -> (u16, u16) {
        (self.0.vid, self.0.pid)
    }

    pub fn to_string(&self) -> String {
        format!(
            "{} - {}",
            self.0
                .manufacturer
                .clone()
                .unwrap_or_else(|| format!("0x{:04x}", self.0.vid)),
            self.0
                .product
                .clone()
                .unwrap_or_else(|| format!("0x{:04x}", self.0.pid))
        )
    }
}

impl From<serialport::UsbPortInfo> for UsbPortInfo {
    fn from(info: serialport::UsbPortInfo) -> Self {
        Self(info)
    }
}

pub fn available_usb_ports() -> Vec<UsbPortInfo> {
    let Ok(ports) = serialport::available_ports() else {
        return vec![];
    };

    ports
        .into_iter()
        .filter_map(|port| match port.port_type {
            SerialPortType::UsbPort(usb) => Some(usb.into()),
            _ => None,
        })
        .collect()
}

pub fn find_usb_port_by_id(id: (u16, u16)) -> Option<UsbPortInfo> {
    available_usb_ports()
        .into_iter()
        .find(|port| port.id() == id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbSerialOutputConfig {
    pub usb_port: UsbPortInfo,

    pub universe: u16,

    #[serde(default)]
    pub enable_rts: bool,
}

pub fn start_serial_output_thread(rx: mpsc::Receiver<DmxData>, config: UsbSerialOutputConfig) {
    thread::spawn(move || {
        let serial_port = serialport::available_ports()
            .expect("No serial devices connected.")
            .into_iter()
            .find(|device| match &device.port_type {
                SerialPortType::UsbPort(usb_port)
                    if config.usb_port.matches(&UsbPortInfo(usb_port.clone())) =>
                {
                    true
                }
                _ => false,
            });

        let Some(serial_port) = serial_port else {
            panic!("No matching serial device found");
        };

        let mut serial = DMXSerial::open(&serial_port.port_name, config.enable_rts).unwrap();
        serial.set_sync();

        loop {
            let recv_result = rx.try_recv();

            if let Ok((send_universe, send_universe_data)) = recv_result {
                if config.universe != send_universe {
                    continue;
                }

                serial.set_channels(send_universe_data);
                serial.update().unwrap();
            } else if recv_result.err().unwrap() == TryRecvError::Disconnected {
                break;
            }
        }
    });
}
