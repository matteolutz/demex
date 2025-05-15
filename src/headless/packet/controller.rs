use std::io::{self, Read};

use super::{DemexProtoDeserialize, DemexProtoSerialize};

use byteorder::{ReadBytesExt, WriteBytesExt};

const HEADLESS_INFO_REQUEST: u8 = 0x01;

#[derive(Debug)]
pub enum DemexProtoControllerPacket {
    HeadlessInfoRequest,
}

impl From<&DemexProtoControllerPacket> for u8 {
    fn from(value: &DemexProtoControllerPacket) -> Self {
        match value {
            DemexProtoControllerPacket::HeadlessInfoRequest => HEADLESS_INFO_REQUEST,
        }
    }
}

impl DemexProtoSerialize for DemexProtoControllerPacket {
    fn serialize(&self, buf: &mut impl std::io::Write) -> std::io::Result<usize> {
        buf.write_u8(self.into())?;
        let bytes_written = 1;

        match self {
            Self::HeadlessInfoRequest => {}
        }

        Ok(bytes_written)
    }
}

impl DemexProtoDeserialize for DemexProtoControllerPacket {
    type Output = DemexProtoControllerPacket;

    fn deserialize(buf: &mut impl Read) -> std::io::Result<Self::Output> {
        match buf.read_u8()? {
            HEADLESS_INFO_REQUEST => Ok(DemexProtoControllerPacket::HeadlessInfoRequest),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid DemexProtoControllerPacket type",
            )),
        }
    }
}
