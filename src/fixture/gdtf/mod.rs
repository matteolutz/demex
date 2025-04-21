use serde::{Deserialize, Serialize};

use crate::ui::utils::color::rgbw_to_rgb;

use super::{
    channel3::{
        attribute::FixtureChannel3Attribute, channel_value::FixtureChannelValue3,
        feature::feature_group::FixtureChannel3FeatureGroup,
    },
    error::FixtureError,
    handler::FixtureTypeList,
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
            .ok_or(FixtureError::GdtfFixtureTypeNotFound(self.fixture_type_id))?;

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

    programmer_values: HashMap<String, FixtureChannelValue3>,
    outputs_values: HashMap<String, FixtureChannelValue3>,

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
            .flatten()
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
            programmer_values: values.clone(),
            outputs_values: values,
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
        &self.programmer_values
    }

    pub fn sources(&self) -> &[FixtureChannelValueSource] {
        &self.sources
    }

    pub fn fixture_type_and_dmx_mode<'a>(
        &self,
        fixture_types: &'a FixtureTypeList,
    ) -> Result<
        (
            &'a gdtf::fixture_type::FixtureType,
            &'a gdtf::dmx_mode::DmxMode,
        ),
        FixtureError,
    > {
        let fixture_type = fixture_types
            .iter()
            .find(|ft| ft.fixture_type_id == self.fixture_type_id)
            .ok_or_else(|| FixtureError::GdtfFixtureTypeNotFound(self.fixture_type_id))?;

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

        for value in self.programmer_values.values_mut() {
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
        fixture_types: &FixtureTypeList,
        feature_group: FixtureChannel3FeatureGroup,
    ) -> Result<Vec<String>, FixtureError> {
        let (fixture_type, dmx_mode) = self.fixture_type_and_dmx_mode(fixture_types)?;

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

    pub fn get_channel<'a>(
        &self,
        fixture_types: &'a FixtureTypeList,
        channel_name: &str,
    ) -> Result<
        (
            &'a gdtf::dmx_mode::DmxChannel,
            &'a gdtf::dmx_mode::LogicalChannel,
        ),
        FixtureError,
    > {
        let (_, dmx_mode) = self.fixture_type_and_dmx_mode(fixture_types)?;

        let dmx_channel = dmx_mode
            .dmx_channel(channel_name)
            .ok_or_else(|| FixtureError::GdtfChannelNotFound(channel_name.to_owned()))?;

        Ok((dmx_channel, &dmx_channel.logical_channels[0]))
    }

    pub fn channels<'a>(
        &self,
        fixture_types: &'a FixtureTypeList,
    ) -> Result<
        impl Iterator<
            Item = (
                &'a gdtf::dmx_mode::DmxChannel,
                &'a gdtf::dmx_mode::LogicalChannel,
            ),
        >,
        FixtureError,
    > {
        let (_, dmx_mode) = self.fixture_type_and_dmx_mode(fixture_types)?;

        Ok(dmx_mode
            .dmx_channels
            .iter()
            .map(|dmx_channel| (dmx_channel, &dmx_channel.logical_channels[0])))
    }

    pub fn channels_for_attribute_matches<'a>(
        &self,
        fixture_types: &'a FixtureTypeList,
        filter: impl Fn(&str) -> bool,
    ) -> Result<
        Vec<(
            &'a gdtf::dmx_mode::DmxChannel,
            &'a gdtf::dmx_mode::LogicalChannel,
            Vec<(usize, &'a str)>,
        )>,
        FixtureError,
    > {
        let (_, dmx_mode) = self.fixture_type_and_dmx_mode(fixture_types)?;

        Ok(dmx_mode
            .dmx_channels
            .iter()
            .map(|dmx_channel| (dmx_channel, &dmx_channel.logical_channels[0]))
            .filter_map(|(dmx_channel, logical_channel)| {
                if filter(logical_channel.attribute.first().unwrap().as_ref()) {
                    return Some((
                        dmx_channel,
                        logical_channel,
                        logical_channel
                            .channel_functions
                            .iter()
                            .enumerate()
                            .map(|(idx, function)| {
                                (idx, function.attribute.first().unwrap().as_ref())
                            })
                            .collect::<Vec<_>>(),
                    ));
                }

                None
            })
            .collect())
    }

    pub fn channels_for_attribute<'a>(
        &self,
        fixture_types: &'a FixtureTypeList,
        attribute: &str,
    ) -> Result<
        Vec<(
            &'a gdtf::dmx_mode::DmxChannel,
            &'a gdtf::dmx_mode::LogicalChannel,
            Vec<(usize, &'a str)>,
        )>,
        FixtureError,
    > {
        self.channels_for_attribute_matches(fixture_types, |attr| attr == attribute)
    }

    pub fn display_color(
        &self,
        fixture_types: &FixtureTypeList,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> Result<[f32; 3], FixtureError> {
        let red = self.get_attribute_display_value(
            fixture_types,
            "ColorAdd_R",
            preset_handler,
            timing_handler,
        )?;
        let green = self.get_attribute_display_value(
            fixture_types,
            "ColorAdd_G",
            preset_handler,
            timing_handler,
        )?;
        let blue = self.get_attribute_display_value(
            fixture_types,
            "ColorAdd_B",
            preset_handler,
            timing_handler,
        )?;
        let white = self
            .get_attribute_display_value(
                fixture_types,
                "ColorAdd_W",
                preset_handler,
                timing_handler,
            )
            .unwrap_or(0.0);

        Ok(rgbw_to_rgb([red, green, blue, white]))
    }

    pub fn get_attribute_display_value(
        &self,
        fixture_types: &FixtureTypeList,
        attribute: &str,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> Result<f32, FixtureError> {
        let (_, dmx_mode) = self.fixture_type_and_dmx_mode(fixture_types)?;

        let (channel, channel_function_idx) = dmx_mode
            .dmx_channels
            .iter()
            .find_map(|dmx_channel| {
                for (idx, channel_function) in dmx_channel.logical_channels[0]
                    .channel_functions
                    .iter()
                    .enumerate()
                {
                    if FixtureChannel3Attribute::attribute_matches(
                        channel_function.attribute.first().unwrap().as_ref(),
                        attribute,
                    ) {
                        return Some((dmx_channel, idx));
                    }
                }
                None
            })
            .ok_or_else(|| FixtureError::GdtfNoChannelForAttributeFound(attribute.to_owned()))?;

        let value = self._get_value(channel)?;

        let (value_channel_function_idx, value_f) = value.get_as_discrete(
            self,
            fixture_types,
            channel.name().as_ref(),
            preset_handler,
            timing_handler,
        );

        if value_channel_function_idx != channel_function_idx {
            return Err(FixtureError::GdtfChannelFunctionMismatch(
                channel_function_idx,
                value_channel_function_idx,
            ));
        }

        Ok(value_f)
    }

    pub fn get_attribute_value(
        &self,
        fixture_types: &FixtureTypeList,
        attribute: &str,
    ) -> Result<FixtureChannelValue3, FixtureError> {
        let (fixture_type, _) = self.fixture_type_and_dmx_mode(fixture_types)?;

        let (channel, _) = self
            .channels(fixture_types)?
            .find(|(_, logical_channel)| {
                logical_channel
                    .attribute(fixture_type)
                    .is_some_and(|fixture_attribute| {
                        fixture_attribute.name.as_ref().unwrap().as_ref() == attribute
                    })
            })
            .ok_or_else(|| FixtureError::GdtfNoChannelForAttributeFound(attribute.to_owned()))?;

        self._get_value(channel)
    }

    pub fn get_value(
        &self,
        fixture_types: &FixtureTypeList,
        channel: &str,
    ) -> Result<FixtureChannelValue3, FixtureError> {
        let (channel, _) = self.get_channel(fixture_types, channel)?;

        self._get_value(channel)
    }

    fn _get_value(
        &self,
        channel: &gdtf::dmx_mode::DmxChannel,
    ) -> Result<FixtureChannelValue3, FixtureError> {
        /*self.sources.get_channel_value(
            fixture_types,
            self,
            channel,
            updatable_handler,
            preset_handler,
            timing_handler,
        )*/

        self.outputs_values
            .get(channel.name().as_ref())
            .ok_or_else(|| {
                FixtureError::GdtfChannelValueNotFound(channel.name().as_ref().to_owned())
            })
            .cloned()
    }

    pub fn get_programmer_value(
        &self,
        channel: &str,
    ) -> Result<&FixtureChannelValue3, FixtureError> {
        self.programmer_values
            .get(channel)
            .ok_or_else(|| FixtureError::GdtfChannelNotFound(channel.to_owned()))
    }

    pub fn set_programmer_value(
        &mut self,
        fixture_types: &FixtureTypeList,
        channel: &str,
        value: FixtureChannelValue3,
    ) -> Result<(), FixtureError> {
        let (fixture_type, _) = self.fixture_type_and_dmx_mode(fixture_types)?;

        let (_, logical_channel) = self.get_channel(fixture_types, channel)?;
        let logical_channel_attribute = logical_channel
            .attribute(fixture_type)
            .ok_or_else(|| FixtureError::GdtfChannelHasNoAttribute(channel.to_owned()))?;

        let programmer_value = self
            .programmer_values
            .get_mut(channel)
            .ok_or_else(|| FixtureError::GdtfChannelNotFound(channel.to_owned()))?;
        *programmer_value = value.clone();

        let linked_value = if value.is_home() {
            value
        } else {
            FixtureChannelValue3::Discrete {
                channel_function_idx: 0,
                value: 0.0,
            }
        };

        if let Some(activation_group) =
            logical_channel_attribute.activation_group(&fixture_type.attribute_definitions)
        {
            for (dmx_channel, _) in
                self.channels(fixture_types)?
                    .filter(|(_, other_logical_channel)| {
                        *other_logical_channel != logical_channel
                            && other_logical_channel
                                .attribute(fixture_type)
                                .and_then(|attribute| {
                                    attribute.activation_group(&fixture_type.attribute_definitions)
                                })
                                .is_some_and(|channel_activation_group| {
                                    channel_activation_group == activation_group
                                })
                    })
            {
                let channel_value = self
                    .programmer_values
                    .get_mut(dmx_channel.name().as_ref())
                    .unwrap();
                if channel_value.is_home() != linked_value.is_home() {
                    *channel_value = linked_value.clone();
                }
            }
        }

        Ok(())
    }

    pub fn update_output_values(
        &mut self,
        fixture_types: &FixtureTypeList,
        preset_handler: &PresetHandler,
        updatable_handler: &UpdatableHandler,
        timing_handler: &TimingHandler,
    ) -> Result<(), FixtureError> {
        let (_, dmx_mode) = self.fixture_type_and_dmx_mode(fixture_types)?;

        for dmx_channel in &dmx_mode.dmx_channels {
            let new_output_value = self.sources.get_channel_value(
                fixture_types,
                self,
                dmx_channel,
                updatable_handler,
                preset_handler,
                timing_handler,
            )?;
            let output_value = self
                .outputs_values
                .get_mut(dmx_channel.name().as_ref())
                .unwrap();
            *output_value = new_output_value;
        }

        Ok(())
    }

    pub fn generate_data_packet(
        &mut self,
        fixture_types: &FixtureTypeList,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
        grand_master: f32,
    ) -> Result<Vec<u8>, FixtureError> {
        /*self.update_output_values(
            fixture_types,
            preset_handler,
            updatable_handler,
            timing_handler,
        )?;*/

        let (_, dmx_mode) = self.fixture_type_and_dmx_mode(fixture_types)?;

        let mut data = vec![0u8; self.address_footprint as usize];

        // loop through dmx_channels
        for dmx_channel in &dmx_mode.dmx_channels {
            let offsets = match &dmx_channel.offset {
                Some(offsets) => offsets,
                None => continue,
            };

            /*let value = self.sources.get_channel_value(
                fixture_types,
                self,
                dmx_channel,
                updatable_handler,
                preset_handler,
                timing_handler,
            )?;*/
            let value = self
                .outputs_values
                .get(dmx_channel.name().as_ref())
                .unwrap();

            let dmx_value = value
                .to_dmx(
                    fixture_types,
                    self,
                    dmx_channel,
                    grand_master,
                    preset_handler,
                    timing_handler,
                )
                .ok_or(FixtureError::GdtfChannelValueNotConvertable(
                    dmx_channel.name().as_ref().to_owned(),
                ))?;
            let mut real_dmx_value = dmx_value.to(offsets.len() as u8);

            for offset in offsets {
                data[*offset as usize - 1] = (real_dmx_value & 0xFF) as u8;
                real_dmx_value >>= 8;
            }
        }

        Ok(data)
    }
}
