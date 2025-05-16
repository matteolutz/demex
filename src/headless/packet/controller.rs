use std::io::{self, Read};

use super::{
    demex_proto_read_bytes, demex_proto_read_u64, demex_proto_write_bytes, demex_proto_write_u64,
    DemexProtoDeserialize, DemexProtoSerialize,
};

use byteorder::{ReadBytesExt, WriteBytesExt};

const HEADLESS_INFO_REQUEST: u8 = 0x01;
const SHOW_FILE_UPDATE: u8 = 0x02;
const SHOW_FILE: u8 = 0x03;
const SYNC: u8 = 0x04;

#[derive(Debug)]
pub enum DemexProtoControllerPacket {
    HeadlessInfoRequest,
    ShowFileUpdate,
    ShowFile { show_file: Vec<u8> },
    Sync { sync: Vec<u8> },
}

impl From<&DemexProtoControllerPacket> for u8 {
    fn from(value: &DemexProtoControllerPacket) -> Self {
        match value {
            DemexProtoControllerPacket::HeadlessInfoRequest => HEADLESS_INFO_REQUEST,
            DemexProtoControllerPacket::ShowFileUpdate => SHOW_FILE_UPDATE,
            DemexProtoControllerPacket::ShowFile { .. } => SHOW_FILE,
            DemexProtoControllerPacket::Sync { .. } => SYNC,
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
                bytes_written += demex_proto_write_u64(buf, show_file.len() as u64)?;
                bytes_written += demex_proto_write_bytes(buf, show_file)?;
            }
            Self::Sync { sync } => {
                bytes_written += demex_proto_write_u64(buf, sync.len() as u64)?;
                bytes_written += demex_proto_write_bytes(buf, sync)?;
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

                Ok(DemexProtoControllerPacket::ShowFile {
                    show_file: show_file_buf,
                })
            }
            SYNC => {
                let len = demex_proto_read_u64(buf)?;
                let mut sync_buf = vec![0; len as usize];
                demex_proto_read_bytes(buf, &mut sync_buf)?;

                Ok(DemexProtoControllerPacket::Sync { sync: sync_buf })
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid DemexProtoControllerPacket type",
            )),
        }
    }
}
