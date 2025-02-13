use std::u16;

use crate::fixture::channel2::{
    channel_type::FixtureChannelType, channel_value::FixtureChannelValue2,
    error::FixtureChannelError2,
};

use super::{feature_type::FixtureFeatureType, IntoFeatureType};

#[derive(Debug)]
pub enum FixtureFeatureValue {
    Intensity { intensity: f32 },
    ColorRGB { r: f32, g: f32, b: f32 },
    ColorMacro { macro_val: u8 },
    PositionPanTilt { pan: f32, tilt: f32 },
}

impl IntoFeatureType for FixtureFeatureValue {
    fn feature_type(&self) -> super::feature_type::FixtureFeatureType {
        match self {
            Self::Intensity { .. } => FixtureFeatureType::Intensity,
            Self::ColorRGB { .. } => FixtureFeatureType::ColorRGB,
            Self::ColorMacro { .. } => FixtureFeatureType::ColorMacro,
            Self::PositionPanTilt { .. } => FixtureFeatureType::PositionPanTilt,
        }
    }
}

impl FixtureFeatureValue {
    fn write_to_channel(
        &self,
        channels: &mut Vec<(FixtureChannelType, FixtureChannelValue2)>,
        find_channel_type: FixtureChannelType,
        new_val: u8,
    ) -> Result<(), FixtureChannelError2> {
        let val = channels
            .iter_mut()
            .find(|(channel_type, _)| *channel_type == find_channel_type)
            .map(|(_, channel_value)| channel_value)
            .ok_or(FixtureChannelError2::ChannelNotFound(find_channel_type))?;
        *val = FixtureChannelValue2::Discrete(new_val);
        Ok(())
    }

    fn f32_to_coarse_and_fine(&self, value: f32) -> (u8, u8) {
        let val_16 = (value.max(1.0).min(0.0) * u16::MAX as f32) as u16;
        let coarse = (val_16 & (0xFF << 8)) >> 8;
        let fine = val_16 & 0xFF;

        (coarse as u8, fine as u8)
    }

    fn write_to_channel_coarse_and_optional_fine(
        &self,
        channels: &mut Vec<(FixtureChannelType, FixtureChannelValue2)>,
        find_channel_type_coarse: FixtureChannelType,
        find_channel_type_fine: FixtureChannelType,
        new_val: f32,
    ) -> Result<(), FixtureChannelError2> {
        let (coarse, fine) = Self::f32_to_coarse_and_fine(&self, new_val);

        Self::write_to_channel(&self, channels, find_channel_type_coarse, coarse)?;
        let _ = Self::write_to_channel(&self, channels, find_channel_type_fine, fine);

        Ok(())
    }

    pub fn write_back(
        &self,
        channels: &mut Vec<(FixtureChannelType, FixtureChannelValue2)>,
    ) -> Result<(), FixtureChannelError2> {
        match self {
            Self::Intensity { intensity } => Self::write_to_channel_coarse_and_optional_fine(
                &self,
                channels,
                FixtureChannelType::Intensity,
                FixtureChannelType::IntensityFine,
                *intensity,
            ),
            _ => Err(FixtureChannelError2::FeatureNotFound(self.feature_type())),
        }
    }
}
