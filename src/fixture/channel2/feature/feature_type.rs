use egui_probe::EguiProbe;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::{
    fixture::{
        channel2::{
            channel_type::FixtureChannelType, channel_value::FixtureChannelValue2,
            error::FixtureChannelError2,
        },
        presets::{preset::FixturePresetId, PresetHandler},
    },
    utils::math::{coarse_fine_to_f32, coarse_to_f32},
};

use super::{
    feature_config::FixtureFeatureConfig, feature_group::DefaultFeatureGroup,
    feature_state::FixtureFeatureDisplayState, feature_value::FixtureFeatureValue, IntoFeatureType,
};

#[derive(
    Debug, Hash, PartialEq, Eq, Copy, Clone, Serialize, Deserialize, EnumIter, EguiProbe, Default,
)]
pub enum FixtureFeatureType {
    #[default]
    Intensity,

    Zoom,
    Focus,

    Shutter,

    ColorRGB,
    ColorMacro,
    PositionPanTilt,

    ToggleFlags,
}

// region private helpers
impl FixtureFeatureType {
    fn find_channel_value(
        channels: &impl Fn(FixtureChannelType) -> Option<FixtureChannelValue2>,
        find_channel_type: FixtureChannelType,
        fixture_id: u32,
        preset_handler: &PresetHandler,
    ) -> Result<u8, FixtureChannelError2> {
        channels(find_channel_type)
            .ok_or(FixtureChannelError2::ChannelNotFound(find_channel_type))
            .and_then(|channel_value| {
                channel_value.to_discrete_value(fixture_id, find_channel_type, preset_handler)
            })
    }

    fn find_coarse(
        channels: &impl Fn(FixtureChannelType) -> Option<FixtureChannelValue2>,
        find_channel_type_coarse: FixtureChannelType,
        fixture_id: u32,
        preset_handler: &PresetHandler,
    ) -> Result<f32, FixtureChannelError2> {
        let coarse = Self::find_channel_value(
            channels,
            find_channel_type_coarse,
            fixture_id,
            preset_handler,
        )?;

        Ok(coarse_to_f32(coarse))
    }

    fn find_coarse_and_optional_fine(
        channels: &impl Fn(FixtureChannelType) -> Option<FixtureChannelValue2>,
        find_channel_type_coarse: FixtureChannelType,
        find_channel_type_fine: FixtureChannelType,
        has_fine: bool,
        fixture_id: u32,
        preset_handler: &PresetHandler,
    ) -> Result<f32, FixtureChannelError2> {
        let coarse = Self::find_channel_value(
            channels,
            find_channel_type_coarse,
            fixture_id,
            preset_handler,
        )?;

        let fine = if has_fine {
            Self::find_channel_value(channels, find_channel_type_fine, fixture_id, preset_handler)?
        } else {
            0
        };

        Ok(coarse_fine_to_f32(coarse, fine))
    }
}

impl FixtureFeatureType {
    pub fn find_feature_config<'a>(
        &self,
        feature_configs: &'a [FixtureFeatureConfig],
    ) -> Result<&'a FixtureFeatureConfig, FixtureChannelError2> {
        feature_configs
            .iter()
            .find(|config| config.feature_type() == *self)
            .ok_or(FixtureChannelError2::FeatureConfigNotFound(*self))
    }

    pub fn get_channel_types(
        &self,
        feature_configs: &[FixtureFeatureConfig],
    ) -> Result<Vec<FixtureChannelType>, FixtureChannelError2> {
        let config = self.find_feature_config(feature_configs)?;
        self._get_channel_types(config)
    }

    fn channel_and_fine(
        channel_type: FixtureChannelType,
        optional_fine: FixtureChannelType,
        is_fine: bool,
    ) -> Vec<FixtureChannelType> {
        if is_fine {
            vec![channel_type, optional_fine]
        } else {
            vec![channel_type]
        }
    }

    fn _get_channel_types(
        &self,
        config: &FixtureFeatureConfig,
    ) -> Result<Vec<FixtureChannelType>, FixtureChannelError2> {
        match (self, config) {
            (Self::Intensity, FixtureFeatureConfig::Intensity { is_fine }) => {
                Ok(Self::channel_and_fine(
                    FixtureChannelType::Intensity,
                    FixtureChannelType::IntensityFine,
                    *is_fine,
                ))
            }
            (
                Self::PositionPanTilt,
                FixtureFeatureConfig::PositionPanTilt { is_fine, has_speed },
            ) => {
                let mut channels = vec![FixtureChannelType::Pan, FixtureChannelType::Tilt];

                if *is_fine {
                    channels.extend([FixtureChannelType::PanFine, FixtureChannelType::TiltFine])
                }

                if *has_speed {
                    channels.push(FixtureChannelType::PanTiltSpeed);
                }

                Ok(channels)
            }
            (Self::ColorRGB, FixtureFeatureConfig::ColorRGB { is_fine }) => {
                let mut channels = vec![
                    FixtureChannelType::Red,
                    FixtureChannelType::Green,
                    FixtureChannelType::Blue,
                ];

                if *is_fine {
                    channels.extend([
                        FixtureChannelType::RedFine,
                        FixtureChannelType::GreenFine,
                        FixtureChannelType::BlueFine,
                    ]);
                }

                Ok(channels)
            }
            (Self::ColorMacro, FixtureFeatureConfig::ColorMacro { .. }) => {
                Ok(vec![FixtureChannelType::ColorMacro])
            }
            (Self::Zoom, FixtureFeatureConfig::Zoom { is_fine }) => Ok(Self::channel_and_fine(
                FixtureChannelType::Zoom,
                FixtureChannelType::ZoomFine,
                *is_fine,
            )),
            (Self::Focus, FixtureFeatureConfig::Focus { is_fine }) => Ok(Self::channel_and_fine(
                FixtureChannelType::Focus,
                FixtureChannelType::FocusFine,
                *is_fine,
            )),
            (Self::Shutter, FixtureFeatureConfig::Shutter) => Ok(vec![FixtureChannelType::Shutter]),
            (Self::ToggleFlags, FixtureFeatureConfig::ToggleFlags { toggle_flags }) => {
                Ok(toggle_flags
                    .iter()
                    .enumerate()
                    .map(|(idx, _)| FixtureChannelType::ToggleFlags(idx))
                    .collect())
            }
            (_, _) => Err(FixtureChannelError2::FeatureNotFound(*self)),
        }
    }

    pub fn get_display_state(
        &self,
        fixture_id: u32,
        feature_configs: &[FixtureFeatureConfig],
        channels: &impl Fn(FixtureChannelType) -> Option<FixtureChannelValue2>,
        preset_handler: &PresetHandler,
    ) -> Result<FixtureFeatureDisplayState, FixtureChannelError2> {
        if self.is_home(feature_configs, channels)? {
            return Ok(FixtureFeatureDisplayState::Home);
        }

        if let Some(preset_id) = self.is_preset(feature_configs, channels)? {
            return Ok(FixtureFeatureDisplayState::Preset(preset_id));
        }

        self.get_value(feature_configs, channels, fixture_id, preset_handler)
            .map(FixtureFeatureDisplayState::FixtureFeatureValue)
    }

    pub fn home(
        &self,
        feature_configs: &[FixtureFeatureConfig],
        channels: &mut [(FixtureChannelType, FixtureChannelValue2)],
    ) -> Result<(), FixtureChannelError2> {
        let config = self.find_feature_config(feature_configs)?;

        for channel_type in self._get_channel_types(config)? {
            let channel_value = channels
                .iter_mut()
                .find(|(find_channel_type, _)| *find_channel_type == channel_type)
                .map(|(_, channel_value)| channel_value)
                .ok_or(FixtureChannelError2::ChannelNotFound(channel_type))?;

            *channel_value = FixtureChannelValue2::Home;
        }

        Ok(())
    }

    pub fn is_home(
        &self,
        feature_configs: &[FixtureFeatureConfig],
        channels: &impl Fn(FixtureChannelType) -> Option<FixtureChannelValue2>,
    ) -> Result<bool, FixtureChannelError2> {
        let config = self.find_feature_config(feature_configs)?;

        for channel_type in self._get_channel_types(config)? {
            if !channels(channel_type)
                .ok_or(FixtureChannelError2::ChannelNotFound(channel_type))?
                .is_home()
            {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn is_preset(
        &self,
        feature_configs: &[FixtureFeatureConfig],
        channels: &impl Fn(FixtureChannelType) -> Option<FixtureChannelValue2>,
    ) -> Result<Option<FixturePresetId>, FixtureChannelError2> {
        let config = self.find_feature_config(feature_configs)?;
        let channel_types = self._get_channel_types(config)?;

        let mut preset_id: Option<FixturePresetId> = None;
        for channel_type in channel_types {
            if let FixtureChannelValue2::Preset(new_preset_id) =
                channels(channel_type).ok_or(FixtureChannelError2::ChannelNotFound(channel_type))?
            {
                if preset_id.is_some() && preset_id.unwrap() != new_preset_id {
                    return Ok(None);
                } else if preset_id.is_none() {
                    preset_id = Some(new_preset_id);
                }
            }
        }

        Ok(preset_id)
    }

    pub fn get_value(
        &self,
        feature_configs: &[FixtureFeatureConfig],
        channels: &impl Fn(FixtureChannelType) -> Option<FixtureChannelValue2>,
        fixture_id: u32,
        preset_handler: &PresetHandler,
    ) -> Result<FixtureFeatureValue, FixtureChannelError2> {
        let feature_config = self.find_feature_config(feature_configs)?;

        match (self, feature_config) {
            (Self::Intensity, FixtureFeatureConfig::Intensity { is_fine }) => {
                Self::find_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Intensity,
                    FixtureChannelType::IntensityFine,
                    *is_fine,
                    fixture_id,
                    preset_handler,
                )
                .map(|intensity| FixtureFeatureValue::Intensity { intensity })
            }
            (Self::Zoom, FixtureFeatureConfig::Zoom { is_fine }) => {
                Self::find_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Zoom,
                    FixtureChannelType::ZoomFine,
                    *is_fine,
                    fixture_id,
                    preset_handler,
                )
                .map(|zoom| FixtureFeatureValue::Zoom { zoom })
            }
            (Self::Focus, FixtureFeatureConfig::Focus { is_fine }) => {
                Self::find_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Focus,
                    FixtureChannelType::FocusFine,
                    *is_fine,
                    fixture_id,
                    preset_handler,
                )
                .map(|focus| FixtureFeatureValue::Focus { focus })
            }
            (
                Self::PositionPanTilt,
                FixtureFeatureConfig::PositionPanTilt { is_fine, has_speed },
            ) => {
                let pan = Self::find_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Pan,
                    FixtureChannelType::PanFine,
                    *is_fine,
                    fixture_id,
                    preset_handler,
                )?;
                let tilt = Self::find_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Tilt,
                    FixtureChannelType::TiltFine,
                    *is_fine,
                    fixture_id,
                    preset_handler,
                )?;

                let pan_tilt_speed = if *has_speed {
                    Some(coarse_to_f32(Self::find_channel_value(
                        channels,
                        FixtureChannelType::PanTiltSpeed,
                        fixture_id,
                        preset_handler,
                    )?))
                } else {
                    None
                };

                Ok(FixtureFeatureValue::PositionPanTilt {
                    pan,
                    tilt,
                    pan_tilt_speed,
                })
            }
            (Self::ColorRGB, FixtureFeatureConfig::ColorRGB { is_fine }) => {
                let r = Self::find_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Red,
                    FixtureChannelType::RedFine,
                    *is_fine,
                    fixture_id,
                    preset_handler,
                )?;

                let g = Self::find_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Green,
                    FixtureChannelType::GreenFine,
                    *is_fine,
                    fixture_id,
                    preset_handler,
                )?;

                let b = Self::find_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Blue,
                    FixtureChannelType::BlueFine,
                    *is_fine,
                    fixture_id,
                    preset_handler,
                )?;

                Ok(FixtureFeatureValue::ColorRGB { r, g, b })
            }
            (Self::ColorMacro, FixtureFeatureConfig::ColorMacro { macros }) => {
                let color_macro_value = Self::find_channel_value(
                    channels,
                    FixtureChannelType::ColorMacro,
                    fixture_id,
                    preset_handler,
                )?;

                let (macro_idx, _) = macros
                    .iter()
                    .find_position(|(macro_value, _)| *macro_value == color_macro_value)
                    .ok_or(FixtureChannelError2::InvalidFeatureValue(*self))?;

                Ok(FixtureFeatureValue::ColorMacro { macro_idx })
            }
            (Self::Shutter, FixtureFeatureConfig::Shutter) => Self::find_coarse(
                channels,
                FixtureChannelType::Shutter,
                fixture_id,
                preset_handler,
            )
            .map(|shutter| FixtureFeatureValue::Shutter { shutter }),
            (Self::ToggleFlags, FixtureFeatureConfig::ToggleFlags { toggle_flags }) => {
                let mut set_flags: Vec<Option<String>> = Vec::new();
                for (idx, toggle_flags) in toggle_flags.iter().enumerate() {
                    let flag_value = Self::find_channel_value(
                        channels,
                        FixtureChannelType::ToggleFlags(idx),
                        fixture_id,
                        preset_handler,
                    )?;

                    set_flags.push(
                        toggle_flags
                            .iter()
                            .find(|(_, value)| **value == flag_value)
                            .map(|(name, _)| name.clone()),
                    );
                }

                Ok(FixtureFeatureValue::ToggleFlags { set_flags })
            }
            (_, _) => Err(FixtureChannelError2::FeatureNotFound(*self)),
        }
    }
}

impl FixtureFeatureType {
    pub fn default_feature_group(&self) -> DefaultFeatureGroup {
        match self {
            Self::Intensity => DefaultFeatureGroup::Intensity,

            Self::ColorMacro | Self::ColorRGB => DefaultFeatureGroup::Color,

            Self::PositionPanTilt => DefaultFeatureGroup::Position,

            Self::Zoom | Self::Focus => DefaultFeatureGroup::Focus,

            Self::Shutter => DefaultFeatureGroup::Beam,

            Self::ToggleFlags => DefaultFeatureGroup::Control,
        }
    }
}
