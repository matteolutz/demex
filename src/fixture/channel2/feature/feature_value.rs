use crate::{
    fixture::channel2::{
        channel_type::FixtureChannelType, channel_value::FixtureChannelValue2,
        error::FixtureChannelError2,
    },
    utils::math::f32_to_coarse_fine,
};

use super::{feature_type::FixtureFeatureType, IntoFeatureType};

#[derive(Debug)]
pub enum FixtureFeatureValue {
    Intensity {
        intensity: f32,
    },
    Zoom {
        zoom: f32,
    },
    ColorRGB {
        r: f32,
        g: f32,
        b: f32,
    },
    ColorMacro {
        macro_val: u8,
    },
    PositionPanTilt {
        pan: f32,
        tilt: f32,
        pan_tilt_speed: Option<f32>,
    },
}

impl IntoFeatureType for FixtureFeatureValue {
    fn feature_type(&self) -> super::feature_type::FixtureFeatureType {
        match self {
            Self::Intensity { .. } => FixtureFeatureType::Intensity,
            Self::Zoom { .. } => FixtureFeatureType::Zoom,
            Self::ColorRGB { .. } => FixtureFeatureType::ColorRGB,
            Self::ColorMacro { .. } => FixtureFeatureType::ColorMacro,
            Self::PositionPanTilt { .. } => FixtureFeatureType::PositionPanTilt,
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
        channels: &mut Vec<(FixtureChannelType, FixtureChannelValue2)>,
        find_channel_type_coarse: FixtureChannelType,
        find_channel_type_fine: FixtureChannelType,
        new_val: f32,
    ) -> Result<(), FixtureChannelError2> {
        let (coarse, fine) = f32_to_coarse_fine(new_val);

        Self::write_to_channel(channels, find_channel_type_coarse, coarse)?;
        let _ = Self::write_to_channel(channels, find_channel_type_fine, fine);

        Ok(())
    }

    pub fn write_back(
        &self,
        channels: &mut Vec<(FixtureChannelType, FixtureChannelValue2)>,
    ) -> Result<(), FixtureChannelError2> {
        match self {
            Self::Intensity { intensity } => Self::write_to_channel_coarse_and_optional_fine(
                channels,
                FixtureChannelType::Intensity,
                FixtureChannelType::IntensityFine,
                *intensity,
            ),
            Self::Zoom { zoom } => Self::write_to_channel_coarse_and_optional_fine(
                channels,
                FixtureChannelType::Zoom,
                FixtureChannelType::ZoomFine,
                *zoom,
            ),
            Self::PositionPanTilt {
                pan,
                tilt,
                pan_tilt_speed,
            } => {
                Self::write_to_channel_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Pan,
                    FixtureChannelType::PanFine,
                    *pan,
                )?;

                Self::write_to_channel_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Tilt,
                    FixtureChannelType::TiltFine,
                    *tilt,
                )?;

                if let Some(pan_tilt_speed) = pan_tilt_speed {
                    Self::write_to_channel(
                        channels,
                        FixtureChannelType::PanTiltSpeed,
                        (*pan_tilt_speed * 255.0) as u8,
                    )?;
                }

                Ok(())
            }
            Self::ColorRGB { r, g, b } => {
                Self::write_to_channel_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Red,
                    FixtureChannelType::RedFine,
                    *r,
                )?;

                Self::write_to_channel_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Green,
                    FixtureChannelType::GreenFine,
                    *g,
                )?;

                Self::write_to_channel_coarse_and_optional_fine(
                    channels,
                    FixtureChannelType::Blue,
                    FixtureChannelType::BlueFine,
                    *b,
                )?;

                Ok(())
            }
            Self::ColorMacro { macro_val } => {
                Self::write_to_channel(channels, FixtureChannelType::ColorMacro, *macro_val)
            }
        }
    }
}
