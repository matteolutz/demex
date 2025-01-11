use std::collections::HashMap;

use error::FixtureChannelError;
use value::{FixtureChannelValue, FixtureChannelValueTrait};

use crate::utils::hash;

use super::presets::PresetHandler;

pub mod error;
pub mod value;

pub const FIXTURE_CHANNEL_INTENSITY_ID: u16 = 0;
pub const FIXTURE_CHANNEL_STROBE: u16 = 1;
pub const FIXTURE_CHANNEL_ZOOM: u16 = 2;
pub const FIXTURE_CHANNEL_COLOR_ID: u16 = 10;
pub const FIXTURE_CHANNEL_POSITION_PAN_TILT_ID: u16 = 20;
pub const FIXTURE_CHANNEL_TOGGLE_FLAGS: u16 = 30;

pub type FixtureId = u32;

#[derive(Debug, Clone, PartialEq)]
pub enum FixtureChannel {
    Intensity(bool, FixtureChannelValue),
    Strobe(FixtureChannelValue),
    Zoom(bool, FixtureChannelValue),
    ColorRGB(bool, FixtureChannelValue),
    PositionPanTilt(bool, FixtureChannelValue),
    Maintenance(String, u16, FixtureChannelValue),
    ToggleFlags(HashMap<String, u8>, FixtureChannelValue),
}

impl FixtureChannel {
    pub fn intensity(is_fine: bool) -> Self {
        FixtureChannel::Intensity(is_fine, FixtureChannelValue::single_default())
    }

    pub fn strobe() -> Self {
        FixtureChannel::Strobe(FixtureChannelValue::single_default())
    }

    pub fn zoom(is_fine: bool) -> Self {
        FixtureChannel::Zoom(is_fine, FixtureChannelValue::single_default())
    }

    pub fn color_rgb(is_fine: bool) -> Self {
        FixtureChannel::ColorRGB(is_fine, FixtureChannelValue::quadruple_default())
    }

    pub fn position_pan_tilt(is_fine: bool) -> Self {
        FixtureChannel::PositionPanTilt(is_fine, FixtureChannelValue::pair_default())
    }

    pub fn maintenance(name: &str) -> Self {
        FixtureChannel::Maintenance(
            name.to_owned(),
            hash::hash(name) as u16,
            FixtureChannelValue::single_default(),
        )
    }

    pub fn toggle_flags(flags: HashMap<String, u8>) -> Self {
        FixtureChannel::ToggleFlags(flags, FixtureChannelValue::toggle_flag_default())
    }
}

impl FixtureChannel {
    pub fn home(&mut self) {
        match self {
            FixtureChannel::Intensity(_, intens) => *intens = FixtureChannelValue::single_default(),
            FixtureChannel::Strobe(strobe) => *strobe = FixtureChannelValue::single_default(),
            FixtureChannel::Zoom(_, zoom) => *zoom = FixtureChannelValue::single_default(),
            FixtureChannel::ColorRGB(_, rgb) => *rgb = FixtureChannelValue::quadruple_default(),
            FixtureChannel::PositionPanTilt(_, position) => {
                *position = FixtureChannelValue::pair_default()
            }
            FixtureChannel::Maintenance(_, _, value) => {
                *value = FixtureChannelValue::single_default()
            }
            FixtureChannel::ToggleFlags(_, value) => {
                *value = FixtureChannelValue::toggle_flag_default()
            }
        }
    }

    pub fn is_home(&self) -> bool {
        match self {
            FixtureChannel::Intensity(_, intens) => intens.is_home(),
            FixtureChannel::Strobe(strobe) => strobe.is_home(),
            FixtureChannel::Zoom(_, zoom) => zoom.is_home(),
            FixtureChannel::ColorRGB(_, color) => color.is_home(),
            FixtureChannel::PositionPanTilt(_, position) => position.is_home(),
            FixtureChannel::Maintenance(_, _, value) => value.is_home(),
            FixtureChannel::ToggleFlags(_, value) => value.is_home(),
        }
    }
}

impl FixtureChannel {
    pub fn address_bandwidth(&self) -> u16 {
        match self {
            FixtureChannel::Intensity(is_fine, _) => {
                if *is_fine {
                    2
                } else {
                    1
                }
            }
            FixtureChannel::Strobe(_) => 1,
            FixtureChannel::Zoom(is_fine, _) => {
                if *is_fine {
                    2
                } else {
                    1
                }
            }
            FixtureChannel::ColorRGB(is_fine, _) => {
                if *is_fine {
                    6
                } else {
                    3
                }
            }
            FixtureChannel::PositionPanTilt(is_fine, _) => {
                if *is_fine {
                    4
                } else {
                    2
                }
            }
            FixtureChannel::Maintenance(_, _, _) => 1,
            FixtureChannel::ToggleFlags(_, _) => 1,
        }
    }

    pub fn type_id(&self) -> u16 {
        match self {
            FixtureChannel::Intensity(_, _) => FIXTURE_CHANNEL_INTENSITY_ID,
            FixtureChannel::Strobe(_) => FIXTURE_CHANNEL_STROBE,
            FixtureChannel::Zoom(_, _) => FIXTURE_CHANNEL_ZOOM,
            FixtureChannel::ColorRGB(_, _) => FIXTURE_CHANNEL_COLOR_ID,
            FixtureChannel::PositionPanTilt(_, _) => FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
            FixtureChannel::Maintenance(_, id, _) => *id,
            FixtureChannel::ToggleFlags(_, _) => FIXTURE_CHANNEL_TOGGLE_FLAGS,
        }
    }

    pub fn name(&self) -> String {
        Self::name_by_id(self.type_id())
    }

    pub fn name_by_id(id: u16) -> String {
        match id {
            FIXTURE_CHANNEL_INTENSITY_ID => "Intensity".to_owned(),
            FIXTURE_CHANNEL_STROBE => "Strobe".to_owned(),
            FIXTURE_CHANNEL_ZOOM => "Zoom".to_owned(),
            FIXTURE_CHANNEL_COLOR_ID => "ColorRGB".to_owned(),
            FIXTURE_CHANNEL_POSITION_PAN_TILT_ID => "PositionPanTilt".to_owned(),
            FIXTURE_CHANNEL_TOGGLE_FLAGS => "ToggleFlags".to_owned(),
            _ => "Unknown".to_owned(),
        }
    }

    fn float_to_coarse_and_fine(val: f32) -> (u8, u8) {
        let coarse = (val * 255.0) as u8;
        let fine = ((val * 255.0 - coarse as f32) * 255.0) as u8;
        (coarse, fine)
    }

    pub fn generate_data_packet(
        &self,
        fixture_id: u32,
        preset_handler: &PresetHandler,
    ) -> Result<Vec<u8>, FixtureChannelError> {
        match self {
            FixtureChannel::Intensity(is_fine, intens) => {
                let (intens_coarse, intens_fine) =
                    Self::float_to_coarse_and_fine(intens.as_single(preset_handler, fixture_id)?);

                if *is_fine {
                    Ok(vec![intens_coarse, intens_fine])
                } else {
                    Ok(vec![intens_coarse])
                }
            }
            FixtureChannel::Strobe(strobe) => Ok(vec![
                (strobe.as_single(preset_handler, fixture_id)? * 255.0) as u8,
            ]),
            FixtureChannel::Zoom(is_fine, zoom) => {
                let (zoom_coarse, zoom_fine) =
                    Self::float_to_coarse_and_fine(zoom.as_single(preset_handler, fixture_id)?);

                if *is_fine {
                    Ok(vec![zoom_coarse, zoom_fine])
                } else {
                    Ok(vec![zoom_coarse])
                }
            }
            FixtureChannel::ColorRGB(is_fine, color) => {
                let [f_r, f_g, f_b, _] =
                    color.as_quadruple(preset_handler, fixture_id, FIXTURE_CHANNEL_COLOR_ID)?;

                let (r, r_fine) = Self::float_to_coarse_and_fine(f_r);
                let (g, g_fine) = Self::float_to_coarse_and_fine(f_g);
                let (b, b_fine) = Self::float_to_coarse_and_fine(f_b);

                if *is_fine {
                    Ok(vec![r, r_fine, g, g_fine, b, b_fine])
                } else {
                    Ok(vec![r, g, b])
                }
            }
            FixtureChannel::PositionPanTilt(is_fine, position) => {
                let [pan_f, tilt_f] = position.as_pair(
                    preset_handler,
                    fixture_id,
                    FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
                )?;

                let (pan, pan_fine) = Self::float_to_coarse_and_fine(pan_f);
                let (tilt, tilt_fine) = Self::float_to_coarse_and_fine(tilt_f);

                if *is_fine {
                    Ok(vec![pan, pan_fine, tilt, tilt_fine])
                } else {
                    Ok(vec![pan, tilt])
                }
            }
            FixtureChannel::Maintenance(_, _, value) => Ok(vec![
                (value.as_single(preset_handler, fixture_id)? * 255.0) as u8,
            ]),
            FixtureChannel::ToggleFlags(flags, set_flag) => {
                let flag_name = set_flag.as_toggle_flag(preset_handler, fixture_id)?;

                let value: u8 = flag_name.map(|f| *flags.get(&f).unwrap_or(&0)).unwrap_or(0);

                Ok(vec![value])
            }
        }
    }
}
