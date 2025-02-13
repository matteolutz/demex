use crate::fixture::{
    channel2::{
        channel_type::FixtureChannelType, channel_value::FixtureChannelValue2,
        error::FixtureChannelError2,
    },
    presets::PresetHandler,
};

use super::{
    feature_config::FixtureFeatureConfig, feature_value::FixtureFeatureValue, IntoFeatureType,
};

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum FixtureFeatureType {
    Intensity,
    ColorRGB,
    ColorMacro,
    PositionPanTilt,
}

impl FixtureFeatureType {
    fn find_channel_value(
        channels: &Vec<(FixtureChannelType, FixtureChannelValue2)>,
        find_channel_type: FixtureChannelType,
        fixture_id: u32,
        preset_handler: &PresetHandler,
    ) -> Result<u8, FixtureChannelError2> {
        channels
            .iter()
            .find(|(channel_type, _)| *channel_type == find_channel_type)
            .ok_or(FixtureChannelError2::ChannelNotFound(find_channel_type))
            .and_then(|(channel_type, channel_value)| {
                channel_value.to_discrete_value(fixture_id, *channel_type, preset_handler)
            })
    }

    fn coarse_fine_to_f32(coarse: u8, fine: u8) -> f32 {
        let combined: u16 = (coarse as u16) << 8 | (fine as u16);
        combined as f32 / u16::MAX as f32
    }

    fn find_coarse_and_optional_fine(
        channels: &Vec<(FixtureChannelType, FixtureChannelValue2)>,
        find_channel_type_coarse: FixtureChannelType,
        find_channel_type_fine: FixtureChannelType,
        fixture_id: u32,
        preset_handler: &PresetHandler,
    ) -> Result<f32, FixtureChannelError2> {
        let coarse = Self::find_channel_value(
            channels,
            find_channel_type_coarse,
            fixture_id,
            preset_handler,
        )?;
        let fine =
            Self::find_channel_value(channels, find_channel_type_fine, fixture_id, preset_handler)
                .unwrap_or(0);

        Ok(Self::coarse_fine_to_f32(coarse, fine))
    }

    fn find_feature_config<'a>(
        &self,
        feature_configs: &'a Vec<FixtureFeatureConfig>,
    ) -> Result<&'a FixtureFeatureConfig, FixtureChannelError2> {
        feature_configs
            .iter()
            .find(|config| config.feature_type() == *self)
            .ok_or(FixtureChannelError2::FeatureConfigNotFound(*self))
    }

    pub fn get_value(
        &self,
        feature_configs: &Vec<FixtureFeatureConfig>,
        channels: &Vec<(FixtureChannelType, FixtureChannelValue2)>,
        fixture_id: u32,
        preset_handler: &PresetHandler,
    ) -> Result<FixtureFeatureValue, FixtureChannelError2> {
        let feature_config = self.find_feature_config(feature_configs)?;

        match (self, feature_config) {
            (Self::Intensity, FixtureFeatureConfig::Intensity) => {
                Ok(FixtureFeatureValue::Intensity {
                    intensity: Self::find_coarse_and_optional_fine(
                        channels,
                        FixtureChannelType::Intensity,
                        FixtureChannelType::IntensityFine,
                        fixture_id,
                        preset_handler,
                    )?,
                })
            }
            _ => Err(FixtureChannelError2::FeatureNotFound(*self)),
        }
    }
}
