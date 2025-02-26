use itertools::Itertools;

use crate::{
    fixture::channel2::{
        channel_type::FixtureChannelType, channel_value::FixtureChannelValue2,
        error::FixtureChannelError2,
    },
    utils::math::{f32_to_coarse, f32_to_coarse_fine},
};

use super::{
    feature_config::FixtureFeatureConfig, feature_type::FixtureFeatureType,
    wheel::WheelFeatureValue, IntoFeatureType,
};

#[derive(Debug, Clone)]
pub enum FixtureFeatureValue {
    SingleValue {
        channel_type: FixtureChannelType,
        value: f32,
    },

    ColorRGB {
        r: f32,
        g: f32,
        b: f32,
    },
    ColorWheel {
        // macro_idx: usize,
        wheel_value: WheelFeatureValue,
    },

    PositionPanTilt {
        pan: f32,
        tilt: f32,
        pan_tilt_speed: Option<f32>,
    },

    ToggleFlags {
        set_flags: Vec<Option<String>>,
    },
}

impl std::fmt::Display for FixtureFeatureValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ColorRGB { r, g, b } => write!(
                f,
                "({:.0}%, {:.0}%, {:.0}%)",
                r * 100.0,
                g * 100.0,
                b * 100.0
            ),
            Self::ColorWheel { wheel_value } => write!(f, "{}", wheel_value),
            Self::PositionPanTilt { pan, tilt, .. } => write!(f, "({:.2}, {:.2})", pan, tilt),

            Self::SingleValue { value, .. } => write!(f, "{:.0}%", *value * 100.0),

            Self::ToggleFlags { set_flags } => {
                write!(
                    f,
                    "{}",
                    set_flags.iter().filter_map(|f| f.as_ref()).join(", ")
                )
            }
        }
    }
}

impl IntoFeatureType for FixtureFeatureValue {
    fn feature_type(&self) -> super::feature_type::FixtureFeatureType {
        match self {
            &Self::SingleValue { channel_type, .. } => {
                FixtureFeatureType::SingleValue { channel_type }
            }

            Self::ColorRGB { .. } => FixtureFeatureType::ColorRGB,
            Self::ColorWheel { .. } => FixtureFeatureType::ColorWheel,

            Self::PositionPanTilt { .. } => FixtureFeatureType::PositionPanTilt,

            Self::ToggleFlags { .. } => FixtureFeatureType::ToggleFlags,
        }
    }
}

impl FixtureFeatureValue {
    fn write_to_channel(
        channels: &mut [(FixtureChannelType, FixtureChannelValue2)],
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

    fn write_to_channel_coarse_and_optional_fine(
        channels: &mut [(FixtureChannelType, FixtureChannelValue2)],
        find_channel_type_coarse: FixtureChannelType,
        find_channel_type_fine: FixtureChannelType,
        has_fine: bool,
        new_val: f32,
    ) -> Result<(), FixtureChannelError2> {
        let (coarse, fine) = f32_to_coarse_fine(new_val);

        Self::write_to_channel(channels, find_channel_type_coarse, coarse)?;
        if has_fine {
            Self::write_to_channel(channels, find_channel_type_fine, fine)?;
        }

        Ok(())
    }

    pub fn write_back(
        &self,
        feature_configs: &[FixtureFeatureConfig],
        channels: &mut [(FixtureChannelType, FixtureChannelValue2)],
    ) -> Result<(), FixtureChannelError2> {
        let config = self.feature_type().find_feature_config(feature_configs)?;

        match (self, config) {
            (
                Self::SingleValue { value, .. },
                FixtureFeatureConfig::SingleValue {
                    channel_type,
                    is_fine,
                },
            ) => {
                if *is_fine {
                    channel_type
                        .get_fine()
                        .ok_or(FixtureChannelError2::FineChannelNotFound(*channel_type))
                        .and_then(|fine_channel_type| {
                            Self::write_to_channel_coarse_and_optional_fine(
                                channels,
                                *channel_type,
                                fine_channel_type,
                                true,
                                *value,
                            )
                        })
                } else {
                    Self::write_to_channel(channels, *channel_type, f32_to_coarse(*value))
                }
            }
            (
                Self::PositionPanTilt {
                    pan,
                    tilt,
                    pan_tilt_speed,
                },
                FixtureFeatureConfig::PositionPanTilt { is_fine, has_speed },
            ) => {
                Self::write_to_channel_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Pan,
                    FixtureChannelType::PanFine,
                    *is_fine,
                    *pan,
                )?;

                Self::write_to_channel_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Tilt,
                    FixtureChannelType::TiltFine,
                    *is_fine,
                    *tilt,
                )?;

                if *has_speed {
                    Self::write_to_channel(
                        channels,
                        FixtureChannelType::PanTiltSpeed,
                        f32_to_coarse(pan_tilt_speed.unwrap_or(0.0)),
                    )?;
                }

                Ok(())
            }
            (Self::ColorRGB { r, g, b }, FixtureFeatureConfig::ColorRGB { is_fine }) => {
                Self::write_to_channel_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Red,
                    FixtureChannelType::RedFine,
                    *is_fine,
                    *r,
                )?;

                Self::write_to_channel_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Green,
                    FixtureChannelType::GreenFine,
                    *is_fine,
                    *g,
                )?;

                Self::write_to_channel_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Blue,
                    FixtureChannelType::BlueFine,
                    *is_fine,
                    *b,
                )?;

                Ok(())
            }
            (
                Self::ColorWheel { wheel_value },
                FixtureFeatureConfig::ColorWheel { wheel_config },
            ) => {
                let macro_value = wheel_config.to_value(wheel_value).ok_or(
                    FixtureChannelError2::InvalidFeatureValue(self.feature_type()),
                )?;

                Self::write_to_channel(channels, FixtureChannelType::ColorMacro, macro_value)
            }
            (
                Self::ToggleFlags { set_flags },
                FixtureFeatureConfig::ToggleFlags { toggle_flags },
            ) => {
                if set_flags.len() != toggle_flags.len() {
                    return Err(FixtureChannelError2::InvalidFeatureValue(
                        self.feature_type(),
                    ));
                }

                for (idx, set_flag) in set_flags.iter().enumerate() {
                    Self::write_to_channel(
                        channels,
                        FixtureChannelType::ToggleFlags(idx),
                        if let Some(set_flag) = set_flag {
                            *toggle_flags[idx].get(set_flag).ok_or(
                                FixtureChannelError2::InvalidFeatureValue(self.feature_type()),
                            )?
                        } else {
                            0
                        },
                    )?;
                }

                Ok(())
            }
            _ => Err(FixtureChannelError2::FeatureNotFound(self.feature_type())),
        }
    }
}
