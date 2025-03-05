use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::{
    fixture::{
        channel2::{
            channel_type::FixtureChannelType, channel_value::FixtureChannelValue2,
            error::FixtureChannelError2,
        },
        presets::{preset::FixturePresetId, PresetHandler},
        timing::TimingHandler,
        Fixture,
    },
    utils::math::{coarse_fine_to_f32, coarse_to_f32},
};

use super::{
    feature_config::FixtureFeatureConfig, feature_state::FixtureFeatureDisplayState,
    feature_value::FixtureFeatureValue, IntoFeatureType,
};

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone, Serialize, Deserialize, EnumIter, EguiProbe)]
pub enum FixtureFeatureType {
    SingleValue { channel_type: FixtureChannelType },

    ColorRGB,
    ColorWheel,

    GoboWheel,

    PositionPanTilt,

    ToggleFlags,
}

impl Default for FixtureFeatureType {
    fn default() -> Self {
        Self::SingleValue {
            channel_type: FixtureChannelType::Intensity,
        }
    }
}

impl std::fmt::Display for FixtureFeatureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SingleValue { channel_type } => write!(f, "{:?}", channel_type),
            _ => write!(f, "{:?}", self),
        }
    }
}

// region private helpers
impl FixtureFeatureType {
    fn find_channel_value(
        channels: &impl Fn(FixtureChannelType) -> Option<FixtureChannelValue2>,
        find_channel_type: FixtureChannelType,
        fixture: &Fixture,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> Result<u8, FixtureChannelError2> {
        channels(find_channel_type)
            .ok_or(FixtureChannelError2::ChannelNotFound(find_channel_type))
            .and_then(|channel_value| {
                channel_value.to_discrete_value(
                    fixture,
                    find_channel_type,
                    preset_handler,
                    timing_handler,
                )
            })
    }

    fn find_coarse(
        channels: &impl Fn(FixtureChannelType) -> Option<FixtureChannelValue2>,
        find_channel_type_coarse: FixtureChannelType,
        fixture: &Fixture,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> Result<f32, FixtureChannelError2> {
        let coarse = Self::find_channel_value(
            channels,
            find_channel_type_coarse,
            fixture,
            preset_handler,
            timing_handler,
        )?;

        Ok(coarse_to_f32(coarse))
    }

    fn find_coarse_and_optional_fine(
        channels: &impl Fn(FixtureChannelType) -> Option<FixtureChannelValue2>,
        find_channel_type_coarse: FixtureChannelType,
        find_channel_type_fine: FixtureChannelType,
        has_fine: bool,
        fixture: &Fixture,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> Result<f32, FixtureChannelError2> {
        let coarse = Self::find_channel_value(
            channels,
            find_channel_type_coarse,
            fixture,
            preset_handler,
            timing_handler,
        )?;

        let fine = if has_fine {
            Self::find_channel_value(
                channels,
                find_channel_type_fine,
                fixture,
                preset_handler,
                timing_handler,
            )?
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

    fn _get_channel_types(
        &self,
        config: &FixtureFeatureConfig,
    ) -> Result<Vec<FixtureChannelType>, FixtureChannelError2> {
        match (self, config) {
            (
                Self::SingleValue { channel_type },
                FixtureFeatureConfig::SingleValue { is_fine, .. },
            ) => {
                if *is_fine {
                    channel_type
                        .get_fine()
                        .ok_or(FixtureChannelError2::FineChannelNotFound(*channel_type))
                        .map(|fine_channel_type| vec![*channel_type, fine_channel_type])
                } else {
                    Ok(vec![*channel_type])
                }
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
            (Self::ColorWheel, FixtureFeatureConfig::ColorWheel { .. }) => {
                Ok(vec![FixtureChannelType::ColorMacro])
            }
            (Self::GoboWheel, FixtureFeatureConfig::GoboWheel { .. }) => {
                Ok(vec![FixtureChannelType::Gobo])
            }
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
        fixture: &Fixture,
        feature_configs: &[FixtureFeatureConfig],
        channels: &impl Fn(FixtureChannelType) -> Option<FixtureChannelValue2>,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> Result<FixtureFeatureDisplayState, FixtureChannelError2> {
        if self.is_home(feature_configs, channels)? {
            return Ok(FixtureFeatureDisplayState::Home);
        }

        if let Some(preset_id) = self.is_preset(feature_configs, channels)? {
            return Ok(FixtureFeatureDisplayState::Preset(preset_id));
        }

        self.get_value(
            feature_configs,
            channels,
            fixture,
            preset_handler,
            timing_handler,
        )
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
        fixture: &Fixture,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> Result<FixtureFeatureValue, FixtureChannelError2> {
        let feature_config = self.find_feature_config(feature_configs)?;

        match (self, feature_config) {
            (
                Self::SingleValue { channel_type },
                FixtureFeatureConfig::SingleValue { is_fine, .. },
            ) => {
                let value = if *is_fine {
                    let fine_channel_type = channel_type
                        .get_fine()
                        .ok_or(FixtureChannelError2::FineChannelNotFound(*channel_type))?;
                    Self::find_coarse_and_optional_fine(
                        channels,
                        *channel_type,
                        fine_channel_type,
                        true,
                        fixture,
                        preset_handler,
                        timing_handler,
                    )?
                } else {
                    Self::find_coarse(
                        channels,
                        *channel_type,
                        fixture,
                        preset_handler,
                        timing_handler,
                    )?
                };

                Ok(FixtureFeatureValue::SingleValue {
                    channel_type: *channel_type,
                    value,
                })
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
                    fixture,
                    preset_handler,
                    timing_handler,
                )?;
                let tilt = Self::find_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Tilt,
                    FixtureChannelType::TiltFine,
                    *is_fine,
                    fixture,
                    preset_handler,
                    timing_handler,
                )?;

                let pan_tilt_speed = if *has_speed {
                    Some(coarse_to_f32(Self::find_channel_value(
                        channels,
                        FixtureChannelType::PanTiltSpeed,
                        fixture,
                        preset_handler,
                        timing_handler,
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
                    fixture,
                    preset_handler,
                    timing_handler,
                )?;

                let g = Self::find_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Green,
                    FixtureChannelType::GreenFine,
                    *is_fine,
                    fixture,
                    preset_handler,
                    timing_handler,
                )?;

                let b = Self::find_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Blue,
                    FixtureChannelType::BlueFine,
                    *is_fine,
                    fixture,
                    preset_handler,
                    timing_handler,
                )?;

                Ok(FixtureFeatureValue::ColorRGB { r, g, b })
            }
            (Self::ColorWheel, FixtureFeatureConfig::ColorWheel { wheel_config }) => {
                let color_macro_value = Self::find_channel_value(
                    channels,
                    FixtureChannelType::ColorMacro,
                    fixture,
                    preset_handler,
                    timing_handler,
                )?;

                let wheel_value = wheel_config
                    .from_value(color_macro_value)
                    .ok_or(FixtureChannelError2::InvalidFeatureValue(*self))?;

                Ok(FixtureFeatureValue::ColorWheel { wheel_value })
            }

            (Self::GoboWheel, FixtureFeatureConfig::GoboWheel { wheel_config }) => {
                let gobo_macro_value = Self::find_channel_value(
                    channels,
                    FixtureChannelType::Gobo,
                    fixture,
                    preset_handler,
                    timing_handler,
                )?;

                let wheel_value = wheel_config
                    .from_value(gobo_macro_value)
                    .ok_or(FixtureChannelError2::InvalidFeatureValue(*self))?;

                Ok(FixtureFeatureValue::GoboWheel { wheel_value })
            }
            (Self::ToggleFlags, FixtureFeatureConfig::ToggleFlags { toggle_flags }) => {
                let mut set_flags: Vec<Option<String>> = Vec::new();
                for (idx, toggle_flags) in toggle_flags.iter().enumerate() {
                    let flag_value = Self::find_channel_value(
                        channels,
                        FixtureChannelType::ToggleFlags(idx),
                        fixture,
                        preset_handler,
                        timing_handler,
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
