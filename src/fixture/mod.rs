use channel::{
    error::FixtureChannelError,
    value::{FixtureChannelDiscreteValue, FixtureChannelValue, FixtureChannelValueTrait},
    FixtureId, SerializableFixtureChannelPatch, FIXTURE_CHANNEL_COLOR_ID,
    FIXTURE_CHANNEL_INTENSITY_ID, FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
};
use presets::PresetHandler;
use serde::{Deserialize, Serialize};

use self::{channel::FixtureChannel, error::FixtureError};

pub mod channel;
pub mod error;
pub mod handler;
pub mod patch;
pub mod presets;
pub mod sequence;

#[derive(Serialize, Deserialize)]
pub struct SerializableFixturePatch {
    id: FixtureId,
    name: String,
    patch: Vec<SerializableFixtureChannelPatch>,
    universe: u16,
    start_address: u16,
}

impl From<Fixture> for SerializableFixturePatch {
    fn from(value: Fixture) -> Self {
        return Self {
            id: value.id,
            name: value.name,
            patch: value
                .patch
                .iter()
                .map(|channel| channel.clone().into())
                .collect(),
            universe: value.universe,
            start_address: value.start_address,
        };
    }
}

impl From<SerializableFixturePatch> for Fixture {
    fn from(value: SerializableFixturePatch) -> Self {
        let patch = value
            .patch
            .into_iter()
            .map(|channel| channel.into())
            .collect();

        Self::new(
            value.id,
            value.name,
            patch,
            value.universe,
            value.start_address,
        )
        .unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct Fixture {
    id: FixtureId,
    name: String,
    patch: Vec<FixtureChannel>,
    universe: u16,
    start_address: u16,
    address_bandwith: u16,
    channel_types: Vec<u16>,
}

impl Serialize for Fixture {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Into::<SerializableFixturePatch>::into(self.clone()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Fixture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        SerializableFixturePatch::deserialize(deserializer).map(Into::<Fixture>::into)
    }
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
        let mut channel_types = Vec::with_capacity(patch.len());

        for channel in &patch {
            if channel_types.contains(&channel.type_id()) {
                return Err(FixtureError::DuplicateChannelType);
            }

            channel_types.push(channel.type_id());
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

    pub fn channel_types(&self) -> &Vec<u16> {
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

    pub fn intensity(&self) -> Result<FixtureChannelValue, FixtureError> {
        match self
            .patch
            .iter()
            .find(|c| matches!(c, FixtureChannel::Intensity(_, _)))
        {
            Some(FixtureChannel::Intensity(_, intens)) => Ok(intens.clone()),
            _ => Err(FixtureError::ChannelNotFound(Some("Intensity".to_string()))),
        }
    }

    pub fn set_intensity(&mut self, value: FixtureChannelValue) -> Result<(), FixtureError> {
        match self
            .patch
            .iter_mut()
            .find(|c| matches!(c, FixtureChannel::Intensity(_, _)))
        {
            Some(FixtureChannel::Intensity(_, intens)) => {
                *intens = value;
                Ok(())
            }
            _ => Err(FixtureError::ChannelNotFound(Some("Intensity".to_string()))),
        }
    }

    pub fn color(&self) -> Result<FixtureChannelValue, FixtureError> {
        match self.patch.iter().find(|c| {
            matches!(
                c,
                FixtureChannel::ColorRGB(_, _) | FixtureChannel::ColorRGBW(_, _)
            )
        }) {
            Some(FixtureChannel::ColorRGB(_, color)) => Ok(color.clone()),
            Some(FixtureChannel::ColorRGBW(_, color)) => Ok(color.clone()),
            _ => Err(FixtureError::ChannelNotFound(Some("Color".to_string()))),
        }
    }

    pub fn set_color(&mut self, value: FixtureChannelValue) -> Result<(), FixtureError> {
        match self.patch.iter_mut().find(|c| {
            matches!(
                c,
                FixtureChannel::ColorRGB(_, _) | FixtureChannel::ColorRGBW(_, _)
            )
        }) {
            Some(FixtureChannel::ColorRGB(_, color)) => {
                *color = value;
                Ok(())
            }
            Some(FixtureChannel::ColorRGBW(_, color)) => {
                *color = value;
                Ok(())
            }
            _ => Err(FixtureError::ChannelNotFound(Some("Color".to_string()))),
        }
    }

    pub fn position_pan_tilt(&self) -> Result<FixtureChannelValue, FixtureError> {
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

    pub fn set_position_pan_tilt(
        &mut self,
        value: FixtureChannelValue,
    ) -> Result<(), FixtureError> {
        match self
            .patch
            .iter_mut()
            .find(|c| matches!(c, FixtureChannel::PositionPanTilt(_, _)))
        {
            Some(FixtureChannel::PositionPanTilt(_, position)) => {
                *position = value;
                Ok(())
            }
            _ => Err(FixtureError::ChannelNotFound(Some(
                "PositionPanTilt".to_string(),
            ))),
        }
    }

    pub fn maintenance(&self, name: &str) -> Result<FixtureChannelValue, FixtureError> {
        match self.patch.iter().find(|c| match c {
            FixtureChannel::Maintenance(n, _, _) => n == name,
            _ => false,
        }) {
            Some(FixtureChannel::Maintenance(_, _, value)) => Ok(value.clone()),
            _ => Err(FixtureError::ChannelNotFound(Some(name.to_string()))),
        }
    }

    pub fn set_mainenance(
        &mut self,
        name: &str,
        value: FixtureChannelValue,
    ) -> Result<(), FixtureError> {
        match self.patch.iter_mut().find(|c| match c {
            FixtureChannel::Maintenance(n, _, _) => n == name,
            _ => false,
        }) {
            Some(FixtureChannel::Maintenance(_, _, val)) => {
                *val = value;
                Ok(())
            }
            _ => Err(FixtureError::ChannelNotFound(Some(name.to_string()))),
        }
    }

    pub fn channel_value(&self, channel_id: u16) -> Result<FixtureChannelValue, FixtureError> {
        match self.patch.iter().find(|c| c.type_id() == channel_id) {
            Some(channel) => match channel {
                FixtureChannel::Intensity(_, intens) => Ok(intens.clone()),
                FixtureChannel::Strobe(strobe) => Ok(strobe.clone()),
                FixtureChannel::Zoom(_, zoom) => Ok(zoom.clone()),
                FixtureChannel::ColorRGB(_, value) | FixtureChannel::ColorRGBW(_, value) => {
                    Ok(value.clone())
                }
                FixtureChannel::PositionPanTilt(_, value) => Ok(value.clone()),
                FixtureChannel::Maintenance(_, _, value) => Ok(value.clone()),
                FixtureChannel::ToggleFlags(_, value) => Ok(value.clone()),
            },
            None => Err(FixtureError::ChannelNotFound(None)),
        }
    }

    pub fn set_channel_value(
        &mut self,
        channel_id: u16,
        value: FixtureChannelValue,
    ) -> Result<(), FixtureError> {
        match self.patch.iter_mut().find(|c| c.type_id() == channel_id) {
            Some(FixtureChannel::Intensity(_, intens)) => {
                *intens = value;
                Ok(())
            }
            Some(FixtureChannel::Strobe(strobe)) => {
                *strobe = value;
                Ok(())
            }
            Some(FixtureChannel::Zoom(_, zoom)) => {
                *zoom = value;
                Ok(())
            }
            Some(FixtureChannel::ColorRGB(_, color)) => {
                *color = value;
                Ok(())
            }
            Some(FixtureChannel::PositionPanTilt(_, position)) => {
                *position = value;
                Ok(())
            }
            Some(FixtureChannel::Maintenance(_, _, maint)) => {
                *maint = value;
                Ok(())
            }
            Some(FixtureChannel::ToggleFlags(_, flags)) => {
                *flags = value;
                Ok(())
            }
            _ => Err(FixtureError::ChannelNotFound(None)),
        }
    }

    pub fn channel_single_value(
        &self,
        channel_id: u16,
        preset_handler: &PresetHandler,
    ) -> Result<f32, FixtureError> {
        match self.patch.iter().find(|c| c.type_id() == channel_id) {
            Some(FixtureChannel::Intensity(_, value)) => Ok(value
                .as_single(preset_handler, self.id)
                .map_err(FixtureError::FixtureChannelError)?),
            Some(FixtureChannel::Strobe(value)) => Ok(value
                .as_single(preset_handler, self.id)
                .map_err(FixtureError::FixtureChannelError)?),
            Some(FixtureChannel::Zoom(_, value)) => Ok(value
                .as_single(preset_handler, self.id)
                .map_err(FixtureError::FixtureChannelError)?),
            Some(FixtureChannel::Maintenance(_, _, value)) => Ok(value
                .as_single(preset_handler, self.id)
                .map_err(FixtureError::FixtureChannelError)?),
            _ => Err(FixtureError::ChannelNotFound(None)),
        }
    }

    pub fn set_channel_single_value(
        &mut self,
        channel_id: u16,
        value: f32,
    ) -> Result<(), FixtureError> {
        match self.patch.iter_mut().find(|c| c.type_id() == channel_id) {
            Some(FixtureChannel::Intensity(_, intens)) => {
                *intens = FixtureChannelValue::Discrete(FixtureChannelDiscreteValue::Single(value));
                Ok(())
            }
            Some(FixtureChannel::Strobe(strobe)) => {
                *strobe = FixtureChannelValue::Discrete(FixtureChannelDiscreteValue::Single(value));
                Ok(())
            }
            Some(FixtureChannel::Zoom(_, zoom)) => {
                *zoom = FixtureChannelValue::Discrete(FixtureChannelDiscreteValue::Single(value));
                Ok(())
            }
            Some(FixtureChannel::Maintenance(_, _, m_value)) => {
                *m_value =
                    FixtureChannelValue::Discrete(FixtureChannelDiscreteValue::Single(value));
                Ok(())
            }
            _ => Err(FixtureError::ChannelNotFound(None)),
        }
    }
}

impl Fixture {
    pub fn home(&mut self) -> Result<(), FixtureError> {
        self.patch.iter_mut().for_each(FixtureChannel::home);

        Ok(())
    }

    pub fn set_toggle_flag(&mut self, flag_name: &str) -> Result<(), FixtureError> {
        match self.patch.iter_mut().find(|c| match c {
            FixtureChannel::ToggleFlags(flags, _) => flags.contains_key(flag_name),
            _ => false,
        }) {
            Some(FixtureChannel::ToggleFlags(_, value)) => {
                *value = FixtureChannelValue::Discrete(FixtureChannelDiscreteValue::ToggleFlag(
                    Some(flag_name.to_owned()),
                ));
                Ok(())
            }
            _ => Err(FixtureError::ChannelNotFound(Some(flag_name.to_string()))),
        }
    }

    pub fn unset_toggle_flags(&mut self) -> Result<(), FixtureError> {
        self.patch.iter_mut().for_each(|c| {
            if let FixtureChannel::ToggleFlags(_, value) = c {
                *value = FixtureChannelValue::toggle_flag_default()
            }
        });

        Ok(())
    }

    pub fn channel_name(&self, type_id: u16) -> Result<String, FixtureError> {
        match self.patch.iter().find(|c| c.type_id() == type_id) {
            Some(channel) => Ok(channel.name().to_string()),
            None => Err(FixtureError::ChannelNotFound(None)),
        }
    }
}

impl Fixture {
    pub fn to_string(&self, preset_handler: &PresetHandler) -> String {
        let mut state = String::new();

        if let Ok(intens) = self
            .intensity()
            .map(|value| value.to_string(preset_handler, FIXTURE_CHANNEL_INTENSITY_ID))
        {
            state.push_str(intens.as_str());
        }

        if let Ok(color) = self
            .color()
            .map(|value| value.to_string(preset_handler, FIXTURE_CHANNEL_COLOR_ID))
        {
            state.push('\n');
            state.push_str(color.as_str());
        }

        if let Ok(position) = self
            .position_pan_tilt()
            .map(|value| value.to_string(preset_handler, FIXTURE_CHANNEL_POSITION_PAN_TILT_ID))
        {
            state.push('\n');
            state.push_str(position.as_str());
        }

        format!(
            "{}\n{} (U{}.{})\n\n{}",
            self.name, self.id, self.universe, self.start_address, state
        )
    }
}
