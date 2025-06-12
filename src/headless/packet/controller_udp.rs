use crate::{
    fixture::channel3::channel_value::FixtureChannelValue3,
    headless::packet::{
        demex_proto_read_string, demex_proto_read_u32, demex_proto_read_u64,
        demex_proto_write_string, demex_proto_write_u32, demex_proto_write_u64,
        DemexProtoDeserialize, DemexProtoSerialize,
    },
};

use byteorder::{ReadBytesExt, WriteBytesExt};

const FIXTURE_OUTPUT_VALUES_UPDATE: u8 = 0x01;

#[derive(Debug)]
pub enum DemexProtoUdpControllerPacket {
    FixtureOutputValuesUpdate {
        values: Vec<(u32, String, FixtureChannelValue3)>,
    },
}

impl From<&DemexProtoUdpControllerPacket> for u8 {
    fn from(value: &DemexProtoUdpControllerPacket) -> Self {
        match value {
            DemexProtoUdpControllerPacket::FixtureOutputValuesUpdate { .. } => {
                FIXTURE_OUTPUT_VALUES_UPDATE
            }
        }
    }
}

impl DemexProtoSerialize for DemexProtoUdpControllerPacket {
    fn serialize(&self, buf: &mut impl std::io::Write) -> std::io::Result<usize> {
        buf.write_u8(self.into())?;
        let mut bytes_written = 1;

        match self {
            Self::FixtureOutputValuesUpdate { values } => {
                bytes_written += demex_proto_write_u64(buf, values.len() as u64)?;

                for (fixture_id, channel_name, value) in values {
                    bytes_written += demex_proto_write_u32(buf, *fixture_id)?;
                    bytes_written += demex_proto_write_string(buf, channel_name)?;
                    bytes_written += value.serialize(buf)?;
                }
            }
        }

        Ok(bytes_written)
    }
}

impl DemexProtoDeserialize for DemexProtoUdpControllerPacket {
    type Output = Self;

    fn deserialize(buf: &mut impl std::io::Read) -> std::io::Result<Self::Output> {
        match buf.read_u8()? {
            FIXTURE_OUTPUT_VALUES_UPDATE => {
                let n_values = demex_proto_read_u64(buf)? as usize;
                let mut values = Vec::with_capacity(n_values);

                for _ in 0..n_values {
                    let fixture_id = demex_proto_read_u32(buf)?;
                    let channel_name = demex_proto_read_string(buf)?;
                    let value = FixtureChannelValue3::deserialize(buf)?;
                    values.push((fixture_id, channel_name, value));
                }

                Ok(Self::FixtureOutputValuesUpdate { values })
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unknown DemexProtoUdpControllerPacket type",
            )),
        }
    }
}
