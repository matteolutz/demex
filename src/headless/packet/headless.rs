use std::io::{self, Read};

use super::{
    demex_proto_read_string, demex_proto_write_string, DemexProtoDeserialize, DemexProtoSerialize,
};

use byteorder::{ReadBytesExt, WriteBytesExt};

const HEADLESS_INFO_RESPONSE: u8 = 0x01;
const SHOW_FILE_REQUEST: u8 = 0x02;
const SYNC_REQUEST: u8 = 0x03;

#[derive(Debug)]
pub enum DemexProtoHeadlessPacket {
    HeadlessInfoResponse { version: String },
    ShowFileRequest,
    SyncRequest,
}

impl From<&DemexProtoHeadlessPacket> for u8 {
    fn from(value: &DemexProtoHeadlessPacket) -> Self {
        match value {
            DemexProtoHeadlessPacket::HeadlessInfoResponse { .. } => HEADLESS_INFO_RESPONSE,
            DemexProtoHeadlessPacket::ShowFileRequest => SHOW_FILE_REQUEST,
            DemexProtoHeadlessPacket::SyncRequest => SYNC_REQUEST,
        }
    }
}

impl DemexProtoSerialize for DemexProtoHeadlessPacket {
    fn serialize(&self, buf: &mut impl std::io::Write) -> std::io::Result<usize> {
        buf.write_u8(self.into())?;
        let mut bytes_written = 1;

        match self {
            Self::HeadlessInfoResponse { version } => {
                bytes_written += demex_proto_write_string(buf, version)?
            }
            Self::ShowFileRequest => {}
            Self::SyncRequest => {}
        }

        Ok(bytes_written)
    }
}

impl DemexProtoDeserialize for DemexProtoHeadlessPacket {
    type Output = DemexProtoHeadlessPacket;

    fn deserialize(buf: &mut impl Read) -> std::io::Result<Self::Output> {
        match buf.read_u8()? {
            HEADLESS_INFO_RESPONSE => {
                let version = demex_proto_read_string(buf)?;
                Ok(DemexProtoHeadlessPacket::HeadlessInfoResponse { version })
            }
            SHOW_FILE_REQUEST => Ok(DemexProtoHeadlessPacket::ShowFileRequest),
            SYNC_REQUEST => Ok(DemexProtoHeadlessPacket::SyncRequest),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid DemexProtoControllerPacket type",
            )),
        }
    }
}
