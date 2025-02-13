use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    fixture::{
        channel2::{
            channel_type::FixtureChannelType, channel_value::FixtureChannelValue2,
            error::FixtureChannelError2,
        },
        presets::PresetHandler,
    },
    utils::math::coarse_fine_to_f32,
};

use super::{
    feature_config::FixtureFeatureConfig, feature_state::FixtureFeatureDisplayState,
    feature_value::FixtureFeatureValue, IntoFeatureType,
};

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum FixtureFeatureType {
    Intensity,
    Zoom,

    ColorRGB,
    ColorMacro,
    PositionPanTilt,
}

impl FixtureFeatureType {
    fn find_channel_value(
        channels: &[(FixtureChannelType, FixtureChannelValue2)],
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

        Ok(coarse_fine_to_f32(coarse, fine))
    }

    fn find_feature_config<'a>(
        &self,
        feature_configs: &'a [FixtureFeatureConfig],
    ) -> Result<&'a FixtureFeatureConfig, FixtureChannelError2> {
        feature_configs
            .iter()
            .find(|config| config.feature_type() == *self)
            .ok_or(FixtureChannelError2::FeatureConfigNotFound(*self))
    }

    pub fn get_channel_types(&self) -> Vec<FixtureChannelType> {
        match self {
            Self::Intensity => vec![
                FixtureChannelType::Intensity,
                FixtureChannelType::IntensityFine,
            ],
            Self::PositionPanTilt => vec![
                FixtureChannelType::Pan,
                FixtureChannelType::PanFine,
                FixtureChannelType::Tilt,
                FixtureChannelType::TiltFine,
                FixtureChannelType::PanTiltSpeed,
            ],
            Self::ColorRGB => vec![
                FixtureChannelType::Red,
                FixtureChannelType::RedFine,
                FixtureChannelType::Green,
                FixtureChannelType::GreenFine,
                FixtureChannelType::Blue,
                FixtureChannelType::BlueFine,
            ],
            Self::ColorMacro => vec![FixtureChannelType::ColorMacro],
            Self::Zoom => vec![FixtureChannelType::Zoom, FixtureChannelType::ZoomFine],
        }
    }

    pub fn get_display_state(
        &self,
        channels: &[(FixtureChannelType, FixtureChannelValue2)],
    ) -> Result<FixtureFeatureDisplayState, FixtureChannelError2> {
        let own_channel_types = self.get_channel_types();

        if own_channel_types.is_empty() {
            // is this right? or should we error?
            return Ok(FixtureFeatureDisplayState::Home);
        }

        let channel_values = own_channel_types
            .iter()
            .filter_map(|channel_type| {
                channels
                    .iter()
                    .find(|(fixture_channel_type, _)| fixture_channel_type == channel_type)
                    .map(|(_, fixture_channel_value)| fixture_channel_value)
            })
            .unique()
            .collect::<Vec<_>>();

        if channel_values.len() == 1 {
            let unique_value = match channel_values[0] {
                FixtureChannelValue2::Preset(preset_id) => {
                    Some(FixtureFeatureDisplayState::Preset(*preset_id))
                }
                FixtureChannelValue2::Home => Some(FixtureFeatureDisplayState::Home),
                _ => None,
            };

            if let Some(unique_value) = unique_value {
                return Ok(unique_value);
            }
        }

        let own_main_chanel_type = own_channel_types[0];
        channels
            .iter()
            .find(|(fixture_channel_type, _)| *fixture_channel_type == own_main_chanel_type)
            .map(|(_, fixture_channel_value)| {
                FixtureFeatureDisplayState::FixtureChannelValue(fixture_channel_value.clone())
            })
            .ok_or(FixtureChannelError2::ChannelNotFound(own_main_chanel_type))
    }

    pub fn home(&self, channels: &mut [(FixtureChannelType, FixtureChannelValue2)]) {
        self.get_channel_types().iter().for_each(|channel_type| {
            if let Some((_, channel_value)) = channels
                .iter_mut()
                .find(|(find_channel_type, _)| find_channel_type == channel_type)
            {
                *channel_value = FixtureChannelValue2::Home;
            }
        });
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
            (Self::Zoom, FixtureFeatureConfig::Zoom) => Ok(FixtureFeatureValue::Zoom {
                zoom: Self::find_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Zoom,
                    FixtureChannelType::ZoomFine,
                    fixture_id,
                    preset_handler,
                )?,
            }),
            (Self::PositionPanTilt, FixtureFeatureConfig::PositionPanTilt { has_speed }) => {
                let pan = Self::find_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Pan,
                    FixtureChannelType::PanFine,
                    fixture_id,
                    preset_handler,
                )?;
                let tilt = Self::find_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Tilt,
                    FixtureChannelType::TiltFine,
                    fixture_id,
                    preset_handler,
                )?;

                let pan_tilt_speed = if *has_speed {
                    Some(
                        Self::find_channel_value(
                            channels,
                            FixtureChannelType::PanTiltSpeed,
                            fixture_id,
                            preset_handler,
                        )? as f32
                            / 255.0,
                    )
                } else {
                    None
                };

                Ok(FixtureFeatureValue::PositionPanTilt {
                    pan,
                    tilt,
                    pan_tilt_speed,
                })
            }
            (Self::ColorRGB, FixtureFeatureConfig::ColorRGB) => {
                let r = Self::find_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Red,
                    FixtureChannelType::RedFine,
                    fixture_id,
                    preset_handler,
                )?;

                let g = Self::find_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Green,
                    FixtureChannelType::GreenFine,
                    fixture_id,
                    preset_handler,
                )?;

                let b = Self::find_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Blue,
                    FixtureChannelType::BlueFine,
                    fixture_id,
                    preset_handler,
                )?;

                Ok(FixtureFeatureValue::ColorRGB { r, g, b })
            }
            _ => Err(FixtureChannelError2::FeatureNotFound(*self)),
        }
    }
}
