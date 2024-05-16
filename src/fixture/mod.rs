use std::collections::BTreeSet;

use self::{channel::FixtureChannel, error::FixtureError};

pub mod channel;
pub mod error;
pub mod handler;

#[derive(Debug)]
pub struct Fixture {
    id: u32,
    name: String,
    patch: Vec<FixtureChannel>,
    universe: u16,
    start_address: u8,
    address_bandwith: u8,
    channel_types: BTreeSet<u16>,
}

impl Fixture {
    pub fn new(
        id: u32,
        name: String,
        patch: Vec<FixtureChannel>,
        universe: u16,
        start_address: u8,
    ) -> Result<Self, FixtureError> {
        // validate, that the patch is not empty
        if patch.is_empty() {
            return Err(FixtureError::EmptyPatch);
        }

        // check, that each channel type is unique
        let mut channel_types = BTreeSet::new();

        for channel in &patch {
            if channel_types.contains(&channel.type_id()) {
                return Err(FixtureError::DuplicateChannelType);
            }

            channel_types.insert(channel.type_id());
        }

        Ok(Self {
            id,
            name,
            address_bandwith: patch
                .iter()
                .fold(0u8, |sum, patch_part| sum + patch_part.address_bandwidth()),
            patch,
            universe,
            start_address,
            channel_types,
        })
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn patch(&self) -> &Vec<FixtureChannel> {
        &self.patch
    }

    pub fn universe(&self) -> u16 {
        self.universe
    }

    pub fn start_address(&self) -> u8 {
        self.start_address
    }

    pub fn address_bandwidth(&self) -> u8 {
        self.address_bandwith
    }

    pub fn channel_types(&self) -> &BTreeSet<u16> {
        &self.channel_types
    }

    pub fn generate_data_packet(&self) -> Vec<u8> {
        self.patch
            .iter()
            .flat_map(|channel| channel.generate_data_packet())
            .collect()
    }
}

impl Fixture {
    pub fn is_home(&self) -> bool {
        self.patch.iter().all(|c| match c {
            FixtureChannel::Intensity(_, intens) => intens.is_none(),
            FixtureChannel::ColorRGB(_, rgb) => rgb.is_none(),
            FixtureChannel::PositionPanTilt(_, pan_tilt) => pan_tilt.is_none(),
            FixtureChannel::Maintenance(_, _, value) => value.is_none(),
        })
    }

    pub fn intensity(&self) -> Result<Option<u8>, FixtureError> {
        match self
            .patch
            .iter()
            .find(|c| matches!(c, FixtureChannel::Intensity(_, _)))
        {
            Some(FixtureChannel::Intensity(_, intens)) => Ok(*intens),
            _ => Err(FixtureError::ChannelNotFound(Some("Intensity".to_string()))),
        }
    }

    pub fn color_rgb(&self) -> Result<Option<[f32; 3]>, FixtureError> {
        match self
            .patch
            .iter()
            .find(|c| matches!(c, FixtureChannel::ColorRGB(_, _)))
        {
            Some(FixtureChannel::ColorRGB(_, rgb)) => Ok(*rgb),
            _ => Err(FixtureError::ChannelNotFound(Some("ColorRGB".to_string()))),
        }
    }

    pub fn maintenance(&self, name: &str) -> Result<Option<u8>, FixtureError> {
        match self.patch.iter().find(|c| match c {
            FixtureChannel::Maintenance(n, _, _) => n == name,
            _ => false,
        }) {
            Some(FixtureChannel::Maintenance(_, _, value)) => Ok(*value),
            _ => Err(FixtureError::ChannelNotFound(Some(name.to_string()))),
        }
    }

    pub fn channel_by_type_id(&self, type_id: u16) -> Result<Option<Vec<u8>>, FixtureError> {
        match self.patch.iter().find(|c| c.type_id() == type_id) {
            Some(channel) => Ok(Some(channel.generate_data_packet())),
            None => Err(FixtureError::ChannelNotFound(None)),
        }
    }
}

impl Fixture {
    pub fn home(&mut self) -> Result<(), FixtureError> {
        // set all channels to none

        for channel in self.patch.iter_mut() {
            match channel {
                FixtureChannel::Intensity(_, intens) => *intens = None,
                FixtureChannel::ColorRGB(_, rgb) => *rgb = None,
                FixtureChannel::PositionPanTilt(_, pan_tilt) => *pan_tilt = None,
                FixtureChannel::Maintenance(_, _, value) => *value = None,
            }
        }

        Ok(())
    }

    pub fn intensity_ref(&mut self) -> Result<&mut Option<u8>, FixtureError> {
        match self
            .patch
            .iter_mut()
            .find(|c| matches!(c, FixtureChannel::Intensity(_, _)))
        {
            Some(FixtureChannel::Intensity(_, intens)) => Ok(intens),
            _ => Err(FixtureError::ChannelNotFound(Some("Intensity".to_string()))),
        }
    }

    pub fn color_rgb_ref(&mut self) -> Result<&mut Option<[f32; 3]>, FixtureError> {
        match self
            .patch
            .iter_mut()
            .find(|c| matches!(c, FixtureChannel::ColorRGB(_, _)))
        {
            Some(FixtureChannel::ColorRGB(_, rgb)) => Ok(rgb),
            _ => Err(FixtureError::ChannelNotFound(Some("ColorRGB".to_string()))),
        }
    }

    pub fn maintenance_ref(&mut self, name: &str) -> Result<&mut Option<u8>, FixtureError> {
        match self.patch.iter_mut().find(|c| match c {
            FixtureChannel::Maintenance(n, _, _) => n == name,
            _ => false,
        }) {
            Some(FixtureChannel::Maintenance(_, _, value_ref)) => Ok(value_ref),
            _ => Err(FixtureError::ChannelNotFound(Some(name.to_string()))),
        }
    }

    pub fn channel_name(&self, type_id: u16) -> Result<String, FixtureError> {
        match self.patch.iter().find(|c| c.type_id() == type_id) {
            Some(channel) => Ok(channel.name().to_string()),
            None => Err(FixtureError::ChannelNotFound(None)),
        }
    }

    pub fn set_channel_by_type_id(
        &mut self,
        type_id: u16,
        data: &[u8],
    ) -> Result<(), FixtureError> {
        match self.patch.iter_mut().find(|c| c.type_id() == type_id) {
            Some(channel) => channel.set_from_data_slice(data),
            None => Err(FixtureError::ChannelNotFound(None)),
        }
    }
}

impl std::fmt::Display for Fixture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut state = String::new();

        if let Ok(intens) = self.intensity() {
            state.push_str(
                format!(
                    "{}",
                    intens.map(|i| i.to_string()).unwrap_or("-".to_string())
                )
                .as_str(),
            );
        }

        if let Ok(rgb) = self.color_rgb() {
            state.push_str(
                format!(
                    "\n{}",
                    rgb.map(|rgb| format!(
                        "{} {} {}",
                        (rgb[0] * 255.0) as u8,
                        (rgb[1] * 255.0) as u8,
                        (rgb[2] * 255.0) as u8
                    ))
                    .unwrap_or("-".to_string())
                )
                .as_str(),
            );
        }

        write!(
            f,
            "{}\n{} (U{}.{})\n\n{}",
            self.name, self.id, self.universe, self.start_address, state
        )
    }
}
