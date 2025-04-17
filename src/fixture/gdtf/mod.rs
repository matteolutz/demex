use std::collections::HashMap;

use gdtf::values::DmxValue;

use super::{
    channel2::channel_value::FixtureChannelValue2, channel3::channel_value::FixtureChannelValue3,
    error::FixtureError, presets::PresetHandler, timing::TimingHandler,
    updatables::UpdatableHandler,
};

pub mod error;

pub struct GdtfFixtureType {}

#[derive(Debug)]
pub struct GdtfFixture<'a> {
    id: u32,
    name: String,

    fixture_type: &'a gdtf::fixture_type::FixtureType,
    dmx_mode: String,

    universe: u16,
    start_address: u16,

    values: Box<[FixtureChannelValue3]>,
}

impl<'a> GdtfFixture<'a> {
    pub fn new(
        id: u32,
        name: String,
        fixture_type: &'a gdtf::fixture_type::FixtureType,
        dmx_mode_name: String,
        universe: u16,
        start_address: u16,
    ) -> Result<Self, FixtureError> {
        let dmx_mode = fixture_type.dmx_mode(&dmx_mode_name).ok_or(
            FixtureError::GdtfFixtureDmxModeNotFound(dmx_mode_name.clone()),
        )?;

        let values = vec![FixtureChannelValue3::Home; dmx_mode.dmx_channels.len()];

        Ok(Self {
            id,
            name,
            fixture_type,
            dmx_mode: dmx_mode_name,
            universe,
            start_address,
            values: values.into_boxed_slice(),
        })
    }

    pub fn generate_data_packet(
        &self,
        // preset_handler: &PresetHandler,
        // updatable_handler: &UpdatableHandler,
        // timing_handler: &TimingHandler,
        _grand_master: f32,
    ) -> Result<Vec<u8>, FixtureError> {
        let dmx_mode = self.fixture_type.dmx_mode(&self.dmx_mode).ok_or(
            FixtureError::GdtfFixtureDmxModeNotFound(self.dmx_mode.clone()),
        )?;

        let max_offset = (dmx_mode
            .dmx_channels
            .iter()
            .flat_map(|dmx_channel| &dmx_channel.offset)
            .flat_map(|offsets| offsets)
            .max()
            .copied()
            .ok_or(FixtureError::GdtfMaxDmxOffsetNotFound)?) as usize;

        let mut data = vec![0u8; max_offset];

        // loop through dmx_channels
        for (idx, dmx_channel) in dmx_mode.dmx_channels.iter().enumerate() {
            let offsets = match &dmx_channel.offset {
                Some(offsets) => offsets,
                None => continue,
            };

            let value = &self.values[idx];

            let dmx_value = value.to_dmx(dmx_mode, dmx_channel, &self.values).ok_or(
                FixtureError::GdtfChannelValueNotConvertable(
                    dmx_channel.name().as_ref().to_owned(),
                ),
            )?;
            let mut real_dmx_value = dmx_value.to(offsets.len() as u8);

            for offset in offsets {
                data[*offset as usize - 1] = (real_dmx_value & 0xFF) as u8;
                real_dmx_value >>= 8;
            }
        }

        Ok(data)
    }
}
