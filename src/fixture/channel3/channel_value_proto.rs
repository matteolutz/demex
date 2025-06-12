use std::io;

use crate::{
    fixture::{
        channel3::{
            channel_value::FixtureChannelValue3,
            feature::feature_group::FixtureChannel3FeatureGroup,
        },
        presets::preset::FixturePresetId,
    },
    headless::packet::{
        demex_proto_read_bytes, demex_proto_read_f32, demex_proto_read_string,
        demex_proto_read_u32, demex_proto_read_u64, demex_proto_write_bytes, demex_proto_write_f32,
        demex_proto_write_string, demex_proto_write_u32, demex_proto_write_u64,
        demex_proto_write_u8, DemexProtoDeserialize, DemexProtoSerialize,
    },
};

use byteorder::ReadBytesExt;

const HOME: u8 = 0x01;
const DISCRETE: u8 = 0x02;
const DISCRETE_SET: u8 = 0x03;
const PRESET: u8 = 0x04;
const MIX: u8 = 0x05;

impl DemexProtoSerialize for FixtureChannelValue3 {
    fn serialize(&self, buf: &mut impl std::io::Write) -> std::io::Result<usize> {
        let mut bytes_written = 0;

        match self {
            Self::Home => {
                bytes_written += demex_proto_write_u8(buf, HOME)?;
            }
            Self::Discrete {
                channel_function_idx,
                value,
            } => {
                bytes_written += demex_proto_write_u8(buf, DISCRETE)?;
                bytes_written += demex_proto_write_u64(buf, *channel_function_idx as u64)?;
                bytes_written += demex_proto_write_f32(buf, *value)?;
            }
            Self::DiscreteSet {
                channel_function_idx,
                channel_set,
            } => {
                bytes_written += demex_proto_write_u8(buf, DISCRETE_SET)?;
                bytes_written += demex_proto_write_u64(buf, *channel_function_idx as u64)?;
                bytes_written += demex_proto_write_string(buf, channel_set)?;
            }
            Self::Preset { id, state } => {
                bytes_written += demex_proto_write_u8(buf, PRESET)?;

                bytes_written += demex_proto_write_u32(buf, id.feature_group.into())?;
                bytes_written += demex_proto_write_u32(buf, id.preset_id)?;

                // TODO: optimize this, maybe also only send discreted values, so that
                // all the effects and preset values are calculated on the controller side
                let mut serialized_state = Vec::new();
                ciborium::into_writer(state, &mut serialized_state).map_err(io::Error::other)?;

                bytes_written += demex_proto_write_u64(buf, serialized_state.len() as u64)?;
                bytes_written += demex_proto_write_bytes(buf, &serialized_state)?;
            }
            Self::Mix { a, b, mix } => {
                bytes_written += demex_proto_write_u8(buf, MIX)?;
                bytes_written += a.serialize(buf)?;
                bytes_written += b.serialize(buf)?;
                bytes_written += demex_proto_write_f32(buf, *mix)?;
            }
        }

        Ok(bytes_written)
    }
}

impl DemexProtoDeserialize for FixtureChannelValue3 {
    type Output = Self;

    fn deserialize(buf: &mut impl io::Read) -> std::io::Result<Self::Output> {
        match buf.read_u8()? {
            HOME => Ok(Self::Home),
            DISCRETE => {
                let channel_function_idx = demex_proto_read_u64(buf)? as usize;
                let value = demex_proto_read_f32(buf)?;

                Ok(Self::Discrete {
                    channel_function_idx,
                    value,
                })
            }
            DISCRETE_SET => {
                let channel_function_idx = demex_proto_read_u64(buf)? as usize;
                let channel_set = demex_proto_read_string(buf)?;

                Ok(Self::DiscreteSet {
                    channel_function_idx,
                    channel_set,
                })
            }
            PRESET => {
                let feature_group: FixtureChannel3FeatureGroup =
                    (demex_proto_read_u32(buf)?).try_into().map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid FixtureChannel3FeatureGroup",
                        )
                    })?;

                let preset_id = demex_proto_read_u32(buf)?;
                let id = FixturePresetId {
                    feature_group,
                    preset_id,
                };

                let preset_state_len = demex_proto_read_u64(buf)? as usize;
                let mut preset_state_buf = vec![0; preset_state_len];
                demex_proto_read_bytes(buf, &mut preset_state_buf)?;

                let state =
                    ciborium::from_reader(&preset_state_buf[..]).map_err(io::Error::other)?;

                Ok(Self::Preset { id, state })
            }
            MIX => {
                let a = FixtureChannelValue3::deserialize(buf)?;
                let b = FixtureChannelValue3::deserialize(buf)?;
                let mix = demex_proto_read_f32(buf)?;

                Ok(Self::Mix {
                    a: Box::new(a),
                    b: Box::new(b),
                    mix,
                })
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid FixtureChannelValue3 type",
            )),
        }
    }
}
