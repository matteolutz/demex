use std::io::{self, Read};

use super::{
    demex_proto_read_string, demex_proto_write_string, DemexProtoDeserialize, DemexProtoSerialize,
};

use byteorder::{ReadBytesExt, WriteBytesExt};

const HEADLESS_INFO_RESPONSE: u8 = 0x01;

#[derive(Debug)]
pub enum DemexProtoHeadlessPacket {
    HeadlessInfoResponse { version: String },
}

impl From<&DemexProtoHeadlessPacket> for u8 {
    fn from(value: &DemexProtoHeadlessPacket) -> Self {
        match value {
            DemexProtoHeadlessPacket::HeadlessInfoResponse { .. } => HEADLESS_INFO_RESPONSE,
        }
    }
}

impl DemexProtoSerialize for DemexProtoHeadlessPacket {
    fn serialize(&self, buf: &mut impl std::io::Write) -> std::io::Result<usize> {
        buf.write_u8(self.into())?;
        let mut bytes_written = 1;

        match self {
            Self::HeadlessInfoResponse { version } => {
                bytes_written += demex_proto_write_string(buf, version)
            }
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
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid DemexProtoControllerPacket type",
            )),
        }
    }
}
