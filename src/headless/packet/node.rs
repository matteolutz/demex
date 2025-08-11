use std::{
    io::{self, Read},
    net::SocketAddr,
};

use crate::headless::packet::{demex_proto_read_sock_addr, demex_proto_write_sock_addr};

use super::{
    demex_proto_read_string, demex_proto_read_u32, demex_proto_write_string, demex_proto_write_u32,
    DemexProtoDeserialize, DemexProtoSerialize,
};

use byteorder::{ReadBytesExt, WriteBytesExt};

const HEADLESS_INFO_RESPONSE: u8 = 0x01;
const SHOW_FILE_REQUEST: u8 = 0x02;
const SYNC_REQUEST: u8 = 0x03;

#[derive(Debug)]
pub enum DemexProtoHeadlessNodePacket {
    HeadlessInfoResponse {
        id: u32,
        version: String,
        udp_addr: SocketAddr,
    },
    ShowFileRequest,
    SyncRequest,
}

impl From<&DemexProtoHeadlessNodePacket> for u8 {
    fn from(value: &DemexProtoHeadlessNodePacket) -> Self {
        match value {
            DemexProtoHeadlessNodePacket::HeadlessInfoResponse { .. } => HEADLESS_INFO_RESPONSE,
            DemexProtoHeadlessNodePacket::ShowFileRequest => SHOW_FILE_REQUEST,
            DemexProtoHeadlessNodePacket::SyncRequest => SYNC_REQUEST,
        }
    }
}

impl DemexProtoSerialize for DemexProtoHeadlessNodePacket {
    fn serialize(&self, buf: &mut impl std::io::Write) -> std::io::Result<usize> {
        buf.write_u8(self.into())?;
        let mut bytes_written = 1;

        match self {
            Self::HeadlessInfoResponse {
                id,
                version,
                udp_addr,
            } => {
                bytes_written += demex_proto_write_u32(buf, *id)?;
                bytes_written += demex_proto_write_string(buf, version)?;
                bytes_written += demex_proto_write_sock_addr(buf, *udp_addr)?;
            }
            Self::ShowFileRequest => {}
            Self::SyncRequest => {}
        }

        Ok(bytes_written)
    }
}

impl DemexProtoDeserialize for DemexProtoHeadlessNodePacket {
    type Output = DemexProtoHeadlessNodePacket;

    fn deserialize(buf: &mut impl Read) -> std::io::Result<Self::Output> {
        match buf.read_u8()? {
            HEADLESS_INFO_RESPONSE => {
                let id = demex_proto_read_u32(buf)?;
                let version = demex_proto_read_string(buf)?;
                let udp_addr = demex_proto_read_sock_addr(buf)?;
                Ok(DemexProtoHeadlessNodePacket::HeadlessInfoResponse {
                    id,
                    version,
                    udp_addr,
                })
            }
            SHOW_FILE_REQUEST => Ok(DemexProtoHeadlessNodePacket::ShowFileRequest),
            SYNC_REQUEST => Ok(DemexProtoHeadlessNodePacket::SyncRequest),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid DemexProtoControllerPacket type",
            )),
        }
    }
}
