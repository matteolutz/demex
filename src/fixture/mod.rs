use std::collections::BTreeSet;

use channel::{
    color::FixtureColorValue, error::FixtureChannelError, position::FixturePositionValue,
    FixtureId, FIXTURE_CHANNEL_COLOR_ID,
};
use presets::PresetHandler;

use self::{channel::FixtureChannel, error::FixtureError};

pub mod channel;
pub mod error;
pub mod handler;
pub mod presets;
pub mod sequence;

#[derive(Debug)]
pub struct Fixture {
    id: FixtureId,
    name: String,
    patch: Vec<FixtureChannel>,
    universe: u16,
    start_address: u16,
    address_bandwith: u16,
    channel_types: BTreeSet<u16>,
}

impl Fixture {
    pub fn new(
        id: FixtureId,
        name: String,
        patch: Vec<FixtureChannel>,
        universe: u16,
        start_address: u16,
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
                .fold(0, |sum, patch_part| sum + patch_part.address_bandwidth()),
            patch,
            universe,
            start_address,
            channel_types,
        })
    }

    pub fn id(&self) -> FixtureId {
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

    pub fn start_address(&self) -> u16 {
        self.start_address
    }

    pub fn address_bandwidth(&self) -> u16 {
        self.address_bandwith
    }

    pub fn channel_types(&self) -> &BTreeSet<u16> {
        &self.channel_types
    }

    pub fn toggle_flags(&self) -> Vec<String> {
        self.patch
            .iter()
            .filter_map(|channel| match channel {
                FixtureChannel::ToggleFlags(flags, _) => {
                    Some(flags.keys().cloned().collect::<Vec<String>>())
                }
                _ => None,
            })
            .flatten()
            .collect()
    }

    pub fn generate_data_packet(
        &self,
        preset_handler: &PresetHandler,
    ) -> Result<Vec<u8>, FixtureChannelError> {
        let mut data = Vec::new();

        for channel in &self.patch {
            data.extend(channel.generate_data_packet(self.id, preset_handler)?);
        }

        Ok(data)
    }
}

impl Fixture {
    pub fn is_home(&self) -> bool {
        self.patch.iter().all(|c| c.is_home())
    }

    pub fn intensity(&self) -> Result<f32, FixtureError> {
        match self
            .patch
            .iter()
            .find(|c| matches!(c, FixtureChannel::Intensity(_, _)))
        {
            Some(FixtureChannel::Intensity(_, intens)) => Ok(*intens),
            _ => Err(FixtureError::ChannelNotFound(Some("Intensity".to_string()))),
        }
    }

    pub fn color(&self) -> Result<FixtureColorValue, FixtureError> {
        match self
            .patch
            .iter()
            .find(|c| matches!(c, FixtureChannel::ColorRGB(_, _)))
        {
            Some(FixtureChannel::ColorRGB(_, color)) => Ok(color.clone()),
            _ => Err(FixtureError::ChannelNotFound(Some("Color".to_string()))),
        }
    }

    pub fn position_pan_tilt(&self) -> Result<FixturePositionValue, FixtureError> {
        match self
            .patch
            .iter()
            .find(|c| matches!(c, FixtureChannel::PositionPanTilt(_, _)))
        {
            Some(FixtureChannel::PositionPanTilt(_, position)) => Ok(position.clone()),
            _ => Err(FixtureError::ChannelNotFound(Some(
                "PositionPanTilt".to_string(),
            ))),
        }
    }

    pub fn maintenance(&self, name: &str) -> Result<u8, FixtureError> {
        match self.patch.iter().find(|c| match c {
            FixtureChannel::Maintenance(n, _, _) => n == name,
            _ => false,
        }) {
            Some(FixtureChannel::Maintenance(_, _, value)) => Ok(*value),
            _ => Err(FixtureError::ChannelNotFound(Some(name.to_string()))),
        }
    }

    pub fn channel_single_value(&self, channel_id: u16) -> Result<f32, FixtureError> {
        match self.patch.iter().find(|c| c.type_id() == channel_id) {
            Some(FixtureChannel::Intensity(_, intens)) => Ok(*intens),
            Some(FixtureChannel::Strobe(strobe)) => Ok(*strobe),
            Some(FixtureChannel::Zoom(_, zoom)) => Ok(*zoom),
            _ => Err(FixtureError::ChannelNotFound(None)),
        }
    }
}

impl Fixture {
    pub fn home(&mut self) -> Result<(), FixtureError> {
        self.patch.iter_mut().for_each(FixtureChannel::home);

        Ok(())
    }

    pub fn intensity_ref(&mut self) -> Result<&mut f32, FixtureError> {
        match self
            .patch
            .iter_mut()
            .find(|c| matches!(c, FixtureChannel::Intensity(_, _)))
        {
            Some(FixtureChannel::Intensity(_, intens)) => Ok(intens),
            _ => Err(FixtureError::ChannelNotFound(Some("Intensity".to_string()))),
        }
    }

    pub fn color_ref(&mut self) -> Result<&mut FixtureColorValue, FixtureError> {
        match self
            .patch
            .iter_mut()
            .find(|c| c.type_id() == FIXTURE_CHANNEL_COLOR_ID)
        {
            Some(FixtureChannel::ColorRGB(_, color)) => Ok(color),
            _ => Err(FixtureError::ChannelNotFound(Some("Color".to_string()))),
        }
    }

    pub fn position_pan_tilt_ref(&mut self) -> Result<&mut FixturePositionValue, FixtureError> {
        match self
            .patch
            .iter_mut()
            .find(|c| matches!(c, FixtureChannel::PositionPanTilt(_, _)))
        {
            Some(FixtureChannel::PositionPanTilt(_, position)) => Ok(position),
            _ => Err(FixtureError::ChannelNotFound(Some(
                "PositionPanTilt".to_string(),
            ))),
        }
    }

    pub fn maintenance_ref(&mut self, name: &str) -> Result<&mut u8, FixtureError> {
        match self.patch.iter_mut().find(|c| match c {
            FixtureChannel::Maintenance(n, _, _) => n == name,
            _ => false,
        }) {
            Some(FixtureChannel::Maintenance(_, _, value_ref)) => Ok(value_ref),
            _ => Err(FixtureError::ChannelNotFound(Some(name.to_string()))),
        }
    }

    pub fn set_toggle_flag(&mut self, flag_name: &str) -> Result<(), FixtureError> {
        match self.patch.iter_mut().find(|c| match c {
            FixtureChannel::ToggleFlags(flags, _) => flags.contains_key(flag_name),
            _ => false,
        }) {
            Some(FixtureChannel::ToggleFlags(_, value)) => {
                *value = Some(flag_name.to_owned());
                Ok(())
            }
            _ => Err(FixtureError::ChannelNotFound(Some(flag_name.to_string()))),
        }
    }

    pub fn unset_toggle_flags(&mut self) -> Result<(), FixtureError> {
        self.patch.iter_mut().for_each(|c| {
            if let FixtureChannel::ToggleFlags(_, value) = c {
                *value = None;
            }
        });

        Ok(())
    }

    pub fn channel_single_value_ref(&mut self, type_id: u16) -> Result<&mut f32, FixtureError> {
        match self.patch.iter_mut().find(|c| c.type_id() == type_id) {
            Some(FixtureChannel::Intensity(_, intens)) => Ok(intens),
            Some(FixtureChannel::Strobe(strobe)) => Ok(strobe),
            Some(FixtureChannel::Zoom(_, zoom)) => Ok(zoom),
            _ => Err(FixtureError::ChannelNotFound(None)),
        }
    }

    pub fn channel_name(&self, type_id: u16) -> Result<String, FixtureError> {
        match self.patch.iter().find(|c| c.type_id() == type_id) {
            Some(channel) => Ok(channel.name().to_string()),
            None => Err(FixtureError::ChannelNotFound(None)),
        }
    }
}

impl Fixture {
    pub fn update_channel(&mut self, channel: &FixtureChannel) -> Result<(), FixtureError> {
        for c in self.patch.iter_mut() {
            if c.type_id() == channel.type_id() {
                *c = channel.clone();
                return Ok(());
            }
        }

        Err(FixtureError::ChannelNotFound(Some(format!(
            "Channel with ID {}",
            channel.type_id()
        ))))
    }

    pub fn update_channels<'a, I>(&mut self, channels: I) -> Result<(), FixtureError>
    where
        I: IntoIterator<Item = &'a FixtureChannel>,
    {
        for channel in channels.into_iter() {
            self.update_channel(channel)?;
        }

        Ok(())
    }
}

impl Fixture {
    pub fn to_string(&self, preset_handler: &PresetHandler) -> String {
        let mut state = String::new();

        if let Ok(intens) = self.intensity() {
            state.push_str(format!("{}%", intens * 100.0).as_str());
        }

        if let Ok(color) = self.color() {
            state.push('\n');
            state.push_str(color.to_string(preset_handler).as_str());
        }

        if let Ok(position) = self.position_pan_tilt() {
            state.push('\n');
            state.push_str(&position.to_string(preset_handler));
        }

        format!(
            "{}\n{} (U{}.{})\n\n{}",
            self.name, self.id, self.universe, self.start_address, state
        )
    }
}
