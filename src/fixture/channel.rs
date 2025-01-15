use std::collections::HashMap;

use error::FixtureChannelError;
use serde::{Deserialize, Serialize};
use value::{FixtureChannelValue, FixtureChannelValueTrait};

use crate::utils::hash;

use super::{presets::PresetHandler, Fixture};

pub mod error;
pub mod value;

pub const FIXTURE_CHANNEL_INTENSITY_ID: u16 = 0;
pub const FIXTURE_CHANNEL_STROBE: u16 = 1;
pub const FIXTURE_CHANNEL_ZOOM: u16 = 2;
pub const FIXTURE_CHANNEL_COLOR_ID: u16 = 10;
pub const FIXTURE_CHANNEL_POSITION_PAN_TILT_ID: u16 = 20;
pub const FIXTURE_CHANNEL_TOGGLE_FLAGS: u16 = 30;

pub type FixtureId = u32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializableFixtureChannelPatch {
    Intensity(bool),
    Strobe,
    Zoom(bool),
    ColorRGB(bool),
    ColorRGBW(bool),
    PositionPanTilt(bool),
    Maintenance(String),
    ToggleFlags(HashMap<String, u8>),
}

impl From<FixtureChannel> for SerializableFixtureChannelPatch {
    fn from(value: FixtureChannel) -> Self {
        match value {
            FixtureChannel::Intensity(is_fine, _) => {
                SerializableFixtureChannelPatch::Intensity(is_fine)
            }
            FixtureChannel::Strobe(_) => SerializableFixtureChannelPatch::Strobe,
            FixtureChannel::Zoom(is_fine, _) => SerializableFixtureChannelPatch::Zoom(is_fine),
            FixtureChannel::ColorRGB(is_fine, _) => {
                SerializableFixtureChannelPatch::ColorRGB(is_fine)
            }
            FixtureChannel::ColorRGBW(is_fine, _) => {
                SerializableFixtureChannelPatch::ColorRGBW(is_fine)
            }
            FixtureChannel::PositionPanTilt(is_fine, _) => {
                SerializableFixtureChannelPatch::PositionPanTilt(is_fine)
            }
            FixtureChannel::Maintenance(name, _, _) => {
                SerializableFixtureChannelPatch::Maintenance(name)
            }
            FixtureChannel::ToggleFlags(flags, _) => {
                SerializableFixtureChannelPatch::ToggleFlags(flags)
            }
        }
    }
}

impl From<SerializableFixtureChannelPatch> for FixtureChannel {
    fn from(value: SerializableFixtureChannelPatch) -> Self {
        match value {
            SerializableFixtureChannelPatch::Intensity(is_fine) => {
                FixtureChannel::intensity(is_fine)
            }
            SerializableFixtureChannelPatch::Strobe => FixtureChannel::strobe(),
            SerializableFixtureChannelPatch::Zoom(is_fine) => FixtureChannel::zoom(is_fine),
            SerializableFixtureChannelPatch::ColorRGB(is_fine) => {
                FixtureChannel::color_rgb(is_fine)
            }
            SerializableFixtureChannelPatch::ColorRGBW(is_fine) => {
                FixtureChannel::color_rgbw(is_fine)
            }
            SerializableFixtureChannelPatch::PositionPanTilt(is_fine) => {
                FixtureChannel::position_pan_tilt(is_fine)
            }
            SerializableFixtureChannelPatch::Maintenance(name) => {
                FixtureChannel::maintenance(&name)
            }
            SerializableFixtureChannelPatch::ToggleFlags(flags) => {
                FixtureChannel::toggle_flags(flags)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FixtureChannel {
    Intensity(bool, FixtureChannelValue),
    Strobe(FixtureChannelValue),
    Zoom(bool, FixtureChannelValue),
    ColorRGB(bool, FixtureChannelValue),
    ColorRGBW(bool, FixtureChannelValue),
    PositionPanTilt(bool, FixtureChannelValue),
    Maintenance(String, u16, FixtureChannelValue),
    ToggleFlags(HashMap<String, u8>, FixtureChannelValue),
}

impl Serialize for FixtureChannel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Into::<SerializableFixtureChannelPatch>::into(self.clone()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for FixtureChannel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        SerializableFixtureChannelPatch::deserialize(deserializer).map(Into::<FixtureChannel>::into)
    }
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

    pub fn color_rgbw(is_fine: bool) -> Self {
        FixtureChannel::ColorRGBW(is_fine, FixtureChannelValue::quadruple_default())
    }

    pub fn position_pan_tilt(is_fine: bool) -> Self {
        FixtureChannel::PositionPanTilt(is_fine, FixtureChannelValue::pair_default())
    }

    pub fn get_maintenance_id(name: &str) -> u16 {
        hash::hash(name) as u16
    }

    pub fn maintenance(name: &str) -> Self {
        FixtureChannel::Maintenance(
            name.to_owned(),
            Self::get_maintenance_id(name),
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
            FixtureChannel::ColorRGBW(_, rgbw) => *rgbw = FixtureChannelValue::quadruple_default(),
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
            FixtureChannel::ColorRGB(_, color) | FixtureChannel::ColorRGBW(_, color) => {
                color.is_home()
            }
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
            FixtureChannel::ColorRGBW(is_fine, _) => {
                if *is_fine {
                    8
                } else {
                    4
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
            FixtureChannel::ColorRGB(_, _) | FixtureChannel::ColorRGBW(_, _) => {
                FIXTURE_CHANNEL_COLOR_ID
            }
            FixtureChannel::PositionPanTilt(_, _) => FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
            FixtureChannel::Maintenance(_, id, _) => *id,
            FixtureChannel::ToggleFlags(_, _) => FIXTURE_CHANNEL_TOGGLE_FLAGS,
        }
    }

    pub fn name(&self) -> String {
        match self {
            FixtureChannel::Maintenance(name, _, _) => name.clone(),
            _ => Self::name_by_id(self.type_id()),
        }
    }

    pub fn name_by_id(id: u16) -> String {
        match id {
            FIXTURE_CHANNEL_INTENSITY_ID => "Intensity".to_owned(),
            FIXTURE_CHANNEL_STROBE => "Strobe".to_owned(),
            FIXTURE_CHANNEL_ZOOM => "Zoom".to_owned(),
            FIXTURE_CHANNEL_COLOR_ID => "Color".to_owned(),
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
        fixture: &Fixture,
        preset_handler: &PresetHandler,
    ) -> Result<Vec<u8>, FixtureChannelError> {
        let fixture_id = fixture.id();

        match self {
            FixtureChannel::Intensity(is_fine, _) => {
                let channel_value = fixture
                    .channel_value(FIXTURE_CHANNEL_INTENSITY_ID, preset_handler)
                    .map_err(FixtureChannelError::FixtureError)?;

                let (intens_coarse, intens_fine) = Self::float_to_coarse_and_fine(
                    channel_value.as_single(preset_handler, fixture.id())?,
                );

                if *is_fine {
                    Ok(vec![intens_coarse, intens_fine])
                } else {
                    Ok(vec![intens_coarse])
                }
            }
            FixtureChannel::Strobe(_) => {
                let channel_value = fixture
                    .channel_value(FIXTURE_CHANNEL_STROBE, preset_handler)
                    .map_err(FixtureChannelError::FixtureError)?;

                Ok(vec![
                    (channel_value.as_single(preset_handler, fixture_id)? * 255.0) as u8,
                ])
            }
            FixtureChannel::Zoom(is_fine, _) => {
                let channel_value = fixture
                    .channel_value(FIXTURE_CHANNEL_ZOOM, preset_handler)
                    .map_err(FixtureChannelError::FixtureError)?;

                let (zoom_coarse, zoom_fine) = Self::float_to_coarse_and_fine(
                    channel_value.as_single(preset_handler, fixture_id)?,
                );

                if *is_fine {
                    Ok(vec![zoom_coarse, zoom_fine])
                } else {
                    Ok(vec![zoom_coarse])
                }
            }
            FixtureChannel::ColorRGB(is_fine, _) => {
                let channel_value = fixture
                    .channel_value(FIXTURE_CHANNEL_COLOR_ID, preset_handler)
                    .map_err(FixtureChannelError::FixtureError)?;

                let [f_r, f_g, f_b, _] = channel_value.as_quadruple(
                    preset_handler,
                    fixture_id,
                    FIXTURE_CHANNEL_COLOR_ID,
                )?;

                let (r, r_fine) = Self::float_to_coarse_and_fine(f_r);
                let (g, g_fine) = Self::float_to_coarse_and_fine(f_g);
                let (b, b_fine) = Self::float_to_coarse_and_fine(f_b);

                if *is_fine {
                    Ok(vec![r, r_fine, g, g_fine, b, b_fine])
                } else {
                    Ok(vec![r, g, b])
                }
            }
            FixtureChannel::ColorRGBW(is_fine, _) => {
                let channel_value = fixture
                    .channel_value(FIXTURE_CHANNEL_COLOR_ID, preset_handler)
                    .map_err(FixtureChannelError::FixtureError)?;

                let [f_r, f_g, f_b, f_w] = channel_value.as_quadruple(
                    preset_handler,
                    fixture_id,
                    FIXTURE_CHANNEL_COLOR_ID,
                )?;

                let (r, r_fine) = Self::float_to_coarse_and_fine(f_r);
                let (g, g_fine) = Self::float_to_coarse_and_fine(f_g);
                let (b, b_fine) = Self::float_to_coarse_and_fine(f_b);
                let (w, w_fine) = Self::float_to_coarse_and_fine(f_w);

                if *is_fine {
                    Ok(vec![r, r_fine, g, g_fine, b, b_fine, w, w_fine])
                } else {
                    Ok(vec![r, g, b, w])
                }
            }
            FixtureChannel::PositionPanTilt(is_fine, _) => {
                let channel_value = fixture
                    .channel_value(FIXTURE_CHANNEL_POSITION_PAN_TILT_ID, preset_handler)
                    .map_err(FixtureChannelError::FixtureError)?;

                let [pan_f, tilt_f] = channel_value.as_pair(
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
            FixtureChannel::Maintenance(_, id, _) => {
                let channel_value = fixture
                    .channel_value(*id, preset_handler)
                    .map_err(FixtureChannelError::FixtureError)?;

                Ok(vec![
                    (channel_value.as_single(preset_handler, fixture_id)? * 255.0) as u8,
                ])
            }
            FixtureChannel::ToggleFlags(flags, _) => {
                let channel_value = fixture
                    .channel_value(FIXTURE_CHANNEL_TOGGLE_FLAGS, preset_handler)
                    .map_err(FixtureChannelError::FixtureError)?;

                let flag_name = channel_value.as_toggle_flag(preset_handler, fixture_id)?;

                let value: u8 = flag_name.map(|f| *flags.get(&f).unwrap_or(&0)).unwrap_or(0);

                Ok(vec![value])
            }
        }
    }
}
