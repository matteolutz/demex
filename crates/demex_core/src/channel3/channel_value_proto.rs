use std::io;

use crate::{
    channel3::{
        channel_value::FixtureChannelValue3, channel_value_discrete::FixtureChannelDiscreteValue,
        feature::feature_group::FixtureChannel3FeatureGroup,
    },
    presets::preset::FixturePresetId,
};

use demex_headless::packet::{
    DemexProtoDeserialize, DemexProtoSerialize, demex_proto_read_bytes, demex_proto_read_f32,
    demex_proto_read_string, demex_proto_read_u32, demex_proto_read_u64, demex_proto_write_bytes,
    demex_proto_write_f32, demex_proto_write_string, demex_proto_write_u8, demex_proto_write_u32,
    demex_proto_write_u64,
};

use byteorder::ReadBytesExt;

const HOME: u8 = 0x01;
const DISCRETE: u8 = 0x02;
const DISCRETE_SET: u8 = 0x03;
const PRESET: u8 = 0x04;
const DISCRETE_MIX: u8 = 0x05;
const MIX: u8 = 0x06;

impl DemexProtoSerialize for FixtureChannelValue3 {
    fn serialize(&self, buf: &mut impl std::io::Write) -> std::io::Result<usize> {
        let mut bytes_written = 0;

        match self {
            Self::Discrete(discrete) => match discrete {
                FixtureChannelDiscreteValue::Home => {
                    bytes_written += demex_proto_write_u8(buf, HOME)?;
                }
                FixtureChannelDiscreteValue::Discrete { value } => {
                    bytes_written += demex_proto_write_u8(buf, DISCRETE)?;
                    bytes_written += demex_proto_write_f32(buf, value.as_f32())?;
                }
                FixtureChannelDiscreteValue::DiscreteSet { channel_set } => {
                    bytes_written += demex_proto_write_u8(buf, DISCRETE_SET)?;
                    bytes_written += demex_proto_write_string(buf, channel_set)?;
                }
                FixtureChannelDiscreteValue::Mix { a, b, mix } => {
                    bytes_written += demex_proto_write_u8(buf, DISCRETE_MIX)?;
                    // TODO: fix this
                    bytes_written += FixtureChannelValue3::Discrete(*a.clone()).serialize(buf)?;
                    bytes_written += FixtureChannelValue3::Discrete(*b.clone()).serialize(buf)?;
                    bytes_written += demex_proto_write_f32(buf, *mix)?;
                }
            },

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
            HOME => Ok(Self::home()),
            DISCRETE => {
                let value = demex_proto_read_f32(buf)?.into();

                Ok(FixtureChannelValue3::Discrete(
                    FixtureChannelDiscreteValue::Discrete { value },
                ))
            }
            DISCRETE_SET => {
                let channel_set = demex_proto_read_string(buf)?;

                Ok(FixtureChannelValue3::Discrete(
                    FixtureChannelDiscreteValue::DiscreteSet { channel_set },
                ))
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
            DISCRETE_MIX => {
                let a = match FixtureChannelValue3::deserialize(buf)? {
                    Self::Discrete(discrete) => Ok(discrete),
                    _ => Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Expected discrete value",
                    )),
                }?;

                let b = match FixtureChannelValue3::deserialize(buf)? {
                    Self::Discrete(discrete) => Ok(discrete),
                    _ => Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Expected discrete value",
                    )),
                }?;

                let mix = demex_proto_read_f32(buf)?;

                Ok(FixtureChannelValue3::Discrete(
                    FixtureChannelDiscreteValue::Mix {
                        a: Box::new(a),
                        b: Box::new(b),
                        mix,
                    },
                ))
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
