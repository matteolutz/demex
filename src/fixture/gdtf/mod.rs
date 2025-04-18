use serde::{Deserialize, Serialize};

use super::{
    channel3::{
        channel_value::FixtureChannelValue3, feature::feature_group::FixtureChannel3FeatureGroup,
        utils::dmx_value_to_f32,
    },
    error::FixtureError,
    handler::FixtureHandler,
    presets::PresetHandler,
    timing::TimingHandler,
    updatables::UpdatableHandler,
    value_source::{FixtureChannelValueSource, FixtureChannelValueSourceTrait},
};
use std::collections::HashMap;

pub mod error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdtfFixturePatch {
    pub id: u32,
    pub name: String,

    pub fixture_type_id: uuid::Uuid,
    pub fixture_type_dmx_mode: String,

    pub universe: u16,
    pub start_address: u16,
}

impl GdtfFixturePatch {
    pub fn into_fixture(
        self,
        fixture_types: &[gdtf::fixture_type::FixtureType],
    ) -> Result<GdtfFixture, FixtureError> {
        let fixture_type = fixture_types
            .iter()
            .find(|ft| ft.fixture_type_id == self.fixture_type_id)
            .ok_or(FixtureError::GdtfFixtureTypeNotFound(
                self.fixture_type_id.clone(),
            ))?;

        GdtfFixture::new(
            self.id,
            self.name,
            fixture_type,
            self.fixture_type_dmx_mode,
            self.universe,
            self.start_address,
        )
    }
}

#[derive(Debug)]
pub struct GdtfFixture {
    id: u32,
    name: String,

    fixture_type_id: uuid::Uuid,
    fixture_type_dmx_mode: String,

    universe: u16,
    start_address: u16,
    address_footprint: u16,

    values: HashMap<String, FixtureChannelValue3>,

    sources: Vec<FixtureChannelValueSource>,
}

impl GdtfFixture {
    pub fn new(
        id: u32,
        name: String,
        fixture_type: &gdtf::fixture_type::FixtureType,
        dmx_mode_name: String,
        universe: u16,
        start_address: u16,
    ) -> Result<Self, FixtureError> {
        let dmx_mode = fixture_type.dmx_mode(&dmx_mode_name).ok_or(
            FixtureError::GdtfFixtureDmxModeNotFound(dmx_mode_name.clone()),
        )?;

        let values: HashMap<String, FixtureChannelValue3> = dmx_mode
            .dmx_channels
            .iter()
            .map(|channel| {
                (
                    channel.name().as_ref().to_owned(),
                    FixtureChannelValue3::Home,
                )
            })
            .collect();

        let address_footprint = (dmx_mode
            .dmx_channels
            .iter()
            .flat_map(|dmx_channel| &dmx_channel.offset)
            .flat_map(|offsets| offsets)
            .max()
            .copied()
            .ok_or(FixtureError::GdtfMaxDmxOffsetNotFound)?) as u16;

        Ok(Self {
            id,
            name,
            fixture_type_id: fixture_type.fixture_type_id,
            fixture_type_dmx_mode: dmx_mode_name,
            universe,
            start_address,
            address_footprint,
            values,
            sources: vec![FixtureChannelValueSource::Programmer],
        })
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn dmx_mode(&self) -> &str {
        &self.fixture_type_dmx_mode
    }

    pub fn universe(&self) -> u16 {
        self.universe
    }

    pub fn start_address(&self) -> u16 {
        self.start_address
    }

    pub fn address_footprint(&self) -> u16 {
        self.address_footprint
    }

    pub fn programmer_values(&self) -> &HashMap<String, FixtureChannelValue3> {
        &self.values
    }

    pub fn sources(&self) -> &[FixtureChannelValueSource] {
        &self.sources
    }

    pub fn fixture_type_and_dmx_mode<'a>(
        &self,
        fixture_handler: &'a FixtureHandler,
    ) -> Result<
        (
            &'a gdtf::fixture_type::FixtureType,
            &'a gdtf::dmx_mode::DmxMode,
        ),
        FixtureError,
    > {
        let fixture_type = fixture_handler.fixture_type(self.fixture_type_id)?;

        let dmx_mode = fixture_type.dmx_mode(&self.fixture_type_dmx_mode).ok_or(
            FixtureError::GdtfFixtureDmxModeNotFound(self.fixture_type_dmx_mode.clone()),
        )?;

        Ok((fixture_type, dmx_mode))
    }
}

impl GdtfFixture {
    pub fn home(&mut self, clear_sources: bool) -> Result<(), FixtureError> {
        if clear_sources {
            // remove every source except the programmer
            self.sources.clear();
            self.sources.push(FixtureChannelValueSource::Programmer);
        }

        for value in self.values.values_mut() {
            *value = FixtureChannelValue3::Home;
        }

        Ok(())
    }

    pub fn push_value_source(&mut self, value_source: FixtureChannelValueSource) {
        self.sources.retain(|source| !source.eq(&value_source));
        self.sources.push(value_source);
    }

    pub fn remove_value_source(&mut self, value_source: FixtureChannelValueSource) {
        self.sources.retain(|source| source != &value_source);
    }

    pub fn get_channels_in_feature_group(
        &self,
        fixture_handler: &FixtureHandler,
        feature_group: FixtureChannel3FeatureGroup,
    ) -> Result<Vec<String>, FixtureError> {
        let (fixture_type, dmx_mode) = self.fixture_type_and_dmx_mode(fixture_handler)?;

        Ok(dmx_mode
            .dmx_channels
            .iter()
            .filter(|channel| {
                let attribute = channel.logical_channels[0].attribute(fixture_type).unwrap();
                let (group_name, _) = attribute.feature.as_ref().unwrap().split_first().unwrap();
                group_name.as_ref() == feature_group.name()
            })
            .map(|channel| channel.name().as_ref().to_owned())
            .collect())
    }

    pub fn get_attribute_display_value(
        &self,
        fixture_handler: &FixtureHandler,
        attribute: &str,
        preset_handler: &PresetHandler,
        updatable_handler: &UpdatableHandler,
        timing_handler: &TimingHandler,
    ) -> Result<f32, FixtureError> {
        let (fixture_type, dmx_mode) = self.fixture_type_and_dmx_mode(fixture_handler)?;

        let channel = dmx_mode
            .dmx_channels
            .iter()
            .find(|dmx_channel| {
                dmx_channel.logical_channels[0]
                    .attribute(fixture_type)
                    .is_some_and(|fixture_attribute| {
                        fixture_attribute.name.as_ref().unwrap().as_ref() == attribute
                    })
            })
            .ok_or_else(|| FixtureError::GdtfNoChannelForAttributeFound(attribute.to_owned()))?;

        let value = self._get_value(
            fixture_handler,
            channel,
            preset_handler,
            updatable_handler,
            timing_handler,
        )?;

        let dmx_value = value
            .to_dmx(fixture_handler, self, channel)
            .ok_or_else(|| FixtureError::GdtfNoChannelForAttributeFound(attribute.to_owned()))?;

        Ok(dmx_value_to_f32(dmx_value))
    }

    pub fn get_attribute_value(
        &self,
        fixture_handler: &FixtureHandler,
        attribute: &str,
        preset_handler: &PresetHandler,
        updatable_handler: &UpdatableHandler,
        timing_handler: &TimingHandler,
    ) -> Result<FixtureChannelValue3, FixtureError> {
        let (fixture_type, dmx_mode) = self.fixture_type_and_dmx_mode(fixture_handler)?;

        let channel = dmx_mode
            .dmx_channels
            .iter()
            .find(|dmx_channel| {
                dmx_channel.logical_channels[0]
                    .attribute(fixture_type)
                    .is_some_and(|fixture_attribute| {
                        fixture_attribute.name.as_ref().unwrap().as_ref() == attribute
                    })
            })
            .ok_or_else(|| FixtureError::GdtfNoChannelForAttributeFound(attribute.to_owned()))?;

        self._get_value(
            fixture_handler,
            channel,
            preset_handler,
            updatable_handler,
            timing_handler,
        )
    }

    pub fn get_value(
        &self,
        fixture_handler: &FixtureHandler,
        channel: &str,
        preset_handler: &PresetHandler,
        updatable_handler: &UpdatableHandler,
        timing_handler: &TimingHandler,
    ) -> Result<FixtureChannelValue3, FixtureError> {
        let (_, dmx_mode) = self.fixture_type_and_dmx_mode(fixture_handler)?;

        let channel = dmx_mode
            .dmx_channels
            .iter()
            .find(|dmx_channel| dmx_channel.name().as_ref() == channel)
            .ok_or_else(|| FixtureError::GdtfChannelNotFound(channel.to_owned()))?;

        self._get_value(
            fixture_handler,
            channel,
            preset_handler,
            updatable_handler,
            timing_handler,
        )
    }

    fn _get_value(
        &self,
        fixture_handler: &FixtureHandler,
        channel: &gdtf::dmx_mode::DmxChannel,
        preset_handler: &PresetHandler,
        updatable_handler: &UpdatableHandler,
        timing_handler: &TimingHandler,
    ) -> Result<FixtureChannelValue3, FixtureError> {
        self.sources.get_channel_value(
            fixture_handler,
            self,
            channel,
            updatable_handler,
            preset_handler,
            timing_handler,
        )
    }

    pub fn get_programmer_value(
        &self,
        channel: &str,
    ) -> Result<&FixtureChannelValue3, FixtureError> {
        self.values
            .get(channel)
            .ok_or_else(|| FixtureError::GdtfChannelNotFound(channel.to_owned()))
    }

    pub fn set_programmer_value(
        &mut self,
        channel: &str,
        value: FixtureChannelValue3,
    ) -> Result<(), FixtureError> {
        let programmer_value = self
            .values
            .get_mut(channel)
            .ok_or_else(|| FixtureError::GdtfChannelNotFound(channel.to_owned()))?;
        *programmer_value = value;
        Ok(())
    }

    pub fn generate_data_packet(
        &self,
        fixture_handler: &FixtureHandler,
        preset_handler: &PresetHandler,
        updatable_handler: &UpdatableHandler,
        timing_handler: &TimingHandler,
        _grand_master: f32,
    ) -> Result<Vec<u8>, FixtureError> {
        let fixture_type = fixture_handler.fixture_type(self.fixture_type_id)?;

        let dmx_mode = fixture_type.dmx_mode(&self.fixture_type_dmx_mode).ok_or(
            FixtureError::GdtfFixtureDmxModeNotFound(self.fixture_type_dmx_mode.clone()),
        )?;

        let mut data = vec![0u8; self.address_footprint as usize];

        // loop through dmx_channels
        for dmx_channel in &dmx_mode.dmx_channels {
            let offsets = match &dmx_channel.offset {
                Some(offsets) => offsets,
                None => continue,
            };

            let value = self.sources.get_channel_value(
                fixture_handler,
                self,
                dmx_channel,
                updatable_handler,
                preset_handler,
                timing_handler,
            )?;

            let dmx_value = value.to_dmx(fixture_handler, self, dmx_channel).ok_or(
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
