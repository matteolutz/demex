use std::io::{self, Read};

use crate::{
    headless::sync::DemexProtoSync, parser::nodes::action::DeferredAction, show::DemexNoUiShow,
};

use super::{
    demex_proto_read_bytes, demex_proto_read_u64, demex_proto_write_bytes, demex_proto_write_u64,
    DemexProtoDeserialize, DemexProtoSerialize,
};

use byteorder::{ReadBytesExt, WriteBytesExt};

const HEADLESS_INFO_REQUEST: u8 = 0x01;
const SHOW_FILE_UPDATE: u8 = 0x02;
const SHOW_FILE: u8 = 0x03;
const SYNC: u8 = 0x04;
const ACTION: u8 = 0x05;

#[derive(Debug)]
pub enum DemexProtoControllerPacket {
    HeadlessInfoRequest,
    ShowFileUpdate,
    ShowFile { show_file: Box<DemexNoUiShow> },
    Sync { sync: Box<DemexProtoSync> },
    Action { action: Box<DeferredAction> },
}

impl From<&DemexProtoControllerPacket> for u8 {
    fn from(value: &DemexProtoControllerPacket) -> Self {
        match value {
            DemexProtoControllerPacket::HeadlessInfoRequest => HEADLESS_INFO_REQUEST,
            DemexProtoControllerPacket::ShowFileUpdate => SHOW_FILE_UPDATE,
            DemexProtoControllerPacket::ShowFile { .. } => SHOW_FILE,
            DemexProtoControllerPacket::Sync { .. } => SYNC,
            DemexProtoControllerPacket::Action { .. } => ACTION,
        }
    }
}

impl DemexProtoSerialize for DemexProtoControllerPacket {
    fn serialize(&self, buf: &mut impl std::io::Write) -> std::io::Result<usize> {
        buf.write_u8(self.into())?;
        let mut bytes_written = 1;

        match self {
            Self::HeadlessInfoRequest => {}
            Self::ShowFileUpdate => {}
            Self::ShowFile { show_file } => {
                let mut serialized_show_file = Vec::new();
                ciborium::into_writer(show_file, &mut serialized_show_file)
                    .map_err(io::Error::other)?;

                bytes_written += demex_proto_write_u64(buf, serialized_show_file.len() as u64)?;
                bytes_written += demex_proto_write_bytes(buf, &serialized_show_file)?;
            }
            Self::Sync { sync } => {
                let mut serialized_sync = Vec::new();
                ciborium::into_writer(sync, &mut serialized_sync).map_err(io::Error::other)?;

                bytes_written += demex_proto_write_u64(buf, serialized_sync.len() as u64)?;
                bytes_written += demex_proto_write_bytes(buf, &serialized_sync)?;
            }
            Self::Action { action } => {
                let mut serialized_action = Vec::new();
                ciborium::into_writer(action, &mut serialized_action).map_err(io::Error::other)?;

                bytes_written += demex_proto_write_u64(buf, serialized_action.len() as u64)?;
                bytes_written += demex_proto_write_bytes(buf, &serialized_action)?;
            }
        }

        Ok(bytes_written)
    }
}

impl DemexProtoDeserialize for DemexProtoControllerPacket {
    type Output = DemexProtoControllerPacket;

    fn deserialize(buf: &mut impl Read) -> std::io::Result<Self::Output> {
        match buf.read_u8()? {
            HEADLESS_INFO_REQUEST => Ok(DemexProtoControllerPacket::HeadlessInfoRequest),
            SHOW_FILE_UPDATE => Ok(DemexProtoControllerPacket::ShowFileUpdate),
            SHOW_FILE => {
                let len = demex_proto_read_u64(buf)?;
                let mut show_file_buf = vec![0; len as usize];
                demex_proto_read_bytes(buf, &mut show_file_buf)?;

                let show_file =
                    ciborium::from_reader(&show_file_buf[..]).map_err(io::Error::other)?;

                Ok(DemexProtoControllerPacket::ShowFile { show_file })
            }
            SYNC => {
                let len = demex_proto_read_u64(buf)?;
                let mut sync_buf = vec![0; len as usize];
                demex_proto_read_bytes(buf, &mut sync_buf)?;

                let sync = ciborium::from_reader(&sync_buf[..]).map_err(io::Error::other)?;

                Ok(DemexProtoControllerPacket::Sync { sync })
            }
            ACTION => {
                let len = demex_proto_read_u64(buf)?;
                let mut action_buf = vec![0; len as usize];
                demex_proto_read_bytes(buf, &mut action_buf)?;

                let action = ciborium::from_reader(&action_buf[..]).map_err(io::Error::other)?;

                Ok(DemexProtoControllerPacket::Action { action })
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid DemexProtoControllerPacket type",
            )),
        }
    }
}
