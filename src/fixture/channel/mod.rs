use std::collections::HashMap;

use error::FixtureChannelError;
use serde::{Deserialize, Serialize};
use value::{FixtureChannelValue, FixtureChannelValueTrait};

use crate::utils::hash;

use super::{presets::PresetHandler, updatables::UpdatableHandler, Fixture};

pub mod error;
pub mod value;
pub mod value_source;

pub const FIXTURE_CHANNEL_INTENSITY_ID: u16 = 0;
pub const FIXTURE_CHANNEL_SHUTTER_ID: u16 = 1;
pub const FIXTURE_CHANNEL_ZOOM_ID: u16 = 2;
pub const FIXTURE_CHANNEL_COLOR_ID: u16 = 10;
pub const FIXTURE_CHANNEL_POSITION_PAN_TILT_ID: u16 = 20;
pub const FIXTURE_CHANNEL_PAN_TILT_SPEED_ID: u16 = 21;
pub const FIXTURE_CHANNEL_TOGGLE_FLAGS: u16 = 30;
pub const FIXTURE_CHANNEL_NO_FUNCTION_ID: u16 = 100;

pub type FixtureId = u32;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FixtureColorChannelMode {
    Rgbw,
    Macro,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializableFixtureChannelPatch {
    Intensity(bool),
    Shutter,
    Zoom(bool),
    ColorRGB(bool),
    ColorRGBW(bool),
    ColorMacro(HashMap<u8, [f32; 4]>),
    PositionPanTilt(bool),
    PanTiltSpeed(bool),
    Maintenance(String),
    ToggleFlags(HashMap<String, u8>),
    NoFunction,
}

impl From<FixtureChannel> for SerializableFixtureChannelPatch {
    fn from(value: FixtureChannel) -> Self {
        match value {
            FixtureChannel::Intensity(is_fine, _) => {
                SerializableFixtureChannelPatch::Intensity(is_fine)
            }
            FixtureChannel::Shutter(_) => SerializableFixtureChannelPatch::Shutter,
            FixtureChannel::Zoom(is_fine, _) => SerializableFixtureChannelPatch::Zoom(is_fine),
            FixtureChannel::ColorRGB(is_fine, _) => {
                SerializableFixtureChannelPatch::ColorRGB(is_fine)
            }
            FixtureChannel::ColorRGBW(is_fine, _) => {
                SerializableFixtureChannelPatch::ColorRGBW(is_fine)
            }
            FixtureChannel::ColorMacro(map, _) => SerializableFixtureChannelPatch::ColorMacro(map),
            FixtureChannel::PositionPanTilt(is_fine, _) => {
                SerializableFixtureChannelPatch::PositionPanTilt(is_fine)
            }
            FixtureChannel::PanTiltSpeed(is_fine, _) => {
                SerializableFixtureChannelPatch::PanTiltSpeed(is_fine)
            }
            FixtureChannel::Maintenance(name, _, _) => {
                SerializableFixtureChannelPatch::Maintenance(name)
            }
            FixtureChannel::ToggleFlags(flags, _) => {
                SerializableFixtureChannelPatch::ToggleFlags(flags)
            }
            FixtureChannel::NoFunction => SerializableFixtureChannelPatch::NoFunction,
        }
    }
}

impl From<SerializableFixtureChannelPatch> for FixtureChannel {
    fn from(value: SerializableFixtureChannelPatch) -> Self {
        match value {
            SerializableFixtureChannelPatch::Intensity(is_fine) => {
                FixtureChannel::intensity(is_fine)
            }
            SerializableFixtureChannelPatch::Shutter => FixtureChannel::strobe(),
            SerializableFixtureChannelPatch::Zoom(is_fine) => FixtureChannel::zoom(is_fine),
            SerializableFixtureChannelPatch::ColorRGB(is_fine) => {
                FixtureChannel::color_rgb(is_fine)
            }
            SerializableFixtureChannelPatch::ColorRGBW(is_fine) => {
                FixtureChannel::color_rgbw(is_fine)
            }
            SerializableFixtureChannelPatch::ColorMacro(map) => {
                FixtureChannel::ColorMacro(map, FixtureChannelValue::any_home())
            }
            SerializableFixtureChannelPatch::PositionPanTilt(is_fine) => {
                FixtureChannel::position_pan_tilt(is_fine)
            }
            SerializableFixtureChannelPatch::PanTiltSpeed(is_fine) => {
                FixtureChannel::pan_tilt_speed(is_fine)
            }
            SerializableFixtureChannelPatch::Maintenance(name) => {
                FixtureChannel::maintenance(&name)
            }
            SerializableFixtureChannelPatch::ToggleFlags(flags) => {
                FixtureChannel::toggle_flags(flags)
            }
            SerializableFixtureChannelPatch::NoFunction => FixtureChannel::NoFunction,
        }
    }
}

impl From<&SerializableFixtureChannelPatch> for FixtureChannel {
    fn from(value: &SerializableFixtureChannelPatch) -> Self {
        match value {
            SerializableFixtureChannelPatch::Intensity(is_fine) => {
                FixtureChannel::intensity(*is_fine)
            }
            SerializableFixtureChannelPatch::Shutter => FixtureChannel::strobe(),
            SerializableFixtureChannelPatch::Zoom(is_fine) => FixtureChannel::zoom(*is_fine),
            SerializableFixtureChannelPatch::ColorRGB(is_fine) => {
                FixtureChannel::color_rgb(*is_fine)
            }
            SerializableFixtureChannelPatch::ColorRGBW(is_fine) => {
                FixtureChannel::color_rgbw(*is_fine)
            }
            SerializableFixtureChannelPatch::ColorMacro(map) => {
                FixtureChannel::ColorMacro(map.clone(), FixtureChannelValue::any_home())
            }
            SerializableFixtureChannelPatch::PositionPanTilt(is_fine) => {
                FixtureChannel::position_pan_tilt(*is_fine)
            }
            SerializableFixtureChannelPatch::PanTiltSpeed(is_fine) => {
                FixtureChannel::pan_tilt_speed(*is_fine)
            }
            SerializableFixtureChannelPatch::Maintenance(name) => FixtureChannel::maintenance(name),
            SerializableFixtureChannelPatch::ToggleFlags(flags) => {
                FixtureChannel::toggle_flags(flags.clone())
            }
            SerializableFixtureChannelPatch::NoFunction => FixtureChannel::NoFunction,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FixtureChannel {
    Intensity(bool, FixtureChannelValue),
    Shutter(FixtureChannelValue),
    Zoom(bool, FixtureChannelValue),
    ColorRGB(bool, FixtureChannelValue),
    ColorRGBW(bool, FixtureChannelValue),
    ColorMacro(HashMap<u8, [f32; 4]>, FixtureChannelValue),
    PositionPanTilt(bool, FixtureChannelValue),
    PanTiltSpeed(bool, FixtureChannelValue),
    Maintenance(String, u16, FixtureChannelValue),
    ToggleFlags(HashMap<String, u8>, FixtureChannelValue),
    NoFunction,
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
        FixtureChannel::Intensity(is_fine, FixtureChannelValue::any_home())
    }

    pub fn strobe() -> Self {
        FixtureChannel::Shutter(FixtureChannelValue::any_home())
    }

    pub fn zoom(is_fine: bool) -> Self {
        FixtureChannel::Zoom(is_fine, FixtureChannelValue::any_home())
    }

    pub fn color_rgb(is_fine: bool) -> Self {
        FixtureChannel::ColorRGB(is_fine, FixtureChannelValue::any_home())
    }

    pub fn color_rgbw(is_fine: bool) -> Self {
        FixtureChannel::ColorRGBW(is_fine, FixtureChannelValue::any_home())
    }

    pub fn position_pan_tilt(is_fine: bool) -> Self {
        FixtureChannel::PositionPanTilt(is_fine, FixtureChannelValue::any_home())
    }

    pub fn pan_tilt_speed(is_fine: bool) -> Self {
        FixtureChannel::PanTiltSpeed(is_fine, FixtureChannelValue::any_home())
    }

    pub fn get_maintenance_id(name: &str) -> u16 {
        hash::hash(name) as u16
    }

    pub fn maintenance(name: &str) -> Self {
        FixtureChannel::Maintenance(
            name.to_owned(),
            Self::get_maintenance_id(name),
            FixtureChannelValue::any_home(),
        )
    }

    pub fn toggle_flags(flags: HashMap<String, u8>) -> Self {
        FixtureChannel::ToggleFlags(flags, FixtureChannelValue::any_home())
    }
}

impl FixtureChannel {
    pub fn home(&mut self) {
        match self {
            FixtureChannel::Intensity(_, intens) => *intens = FixtureChannelValue::any_home(),
            FixtureChannel::Shutter(strobe) => *strobe = FixtureChannelValue::any_home(),
            FixtureChannel::Zoom(_, zoom) => *zoom = FixtureChannelValue::any_home(),
            FixtureChannel::ColorRGB(_, rgb) => *rgb = FixtureChannelValue::any_home(),
            FixtureChannel::ColorRGBW(_, rgbw) => *rgbw = FixtureChannelValue::any_home(),
            FixtureChannel::ColorMacro(_, value) => *value = FixtureChannelValue::any_home(),
            FixtureChannel::PositionPanTilt(_, position) => {
                *position = FixtureChannelValue::any_home()
            }
            FixtureChannel::PanTiltSpeed(_, speed) => *speed = FixtureChannelValue::any_home(),
            FixtureChannel::Maintenance(_, _, value) => *value = FixtureChannelValue::any_home(),
            FixtureChannel::ToggleFlags(_, value) => *value = FixtureChannelValue::any_home(),
            FixtureChannel::NoFunction => {}
        }
    }

    pub fn is_home(&self) -> bool {
        match self {
            FixtureChannel::Intensity(_, intens) => intens.is_home(),
            FixtureChannel::Shutter(strobe) => strobe.is_home(),
            FixtureChannel::Zoom(_, zoom) => zoom.is_home(),
            FixtureChannel::ColorRGB(_, color) | FixtureChannel::ColorRGBW(_, color) => {
                color.is_home()
            }
            FixtureChannel::ColorMacro(_, value) => value.is_home(),
            FixtureChannel::PositionPanTilt(_, position) => position.is_home(),
            FixtureChannel::PanTiltSpeed(_, speed) => speed.is_home(),
            FixtureChannel::Maintenance(_, _, value) => value.is_home(),
            FixtureChannel::ToggleFlags(_, value) => value.is_home(),
            FixtureChannel::NoFunction => true,
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
            FixtureChannel::Shutter(_) => 1,
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
            FixtureChannel::ColorMacro(_, _) => 1,
            FixtureChannel::PositionPanTilt(is_fine, _) => {
                if *is_fine {
                    4
                } else {
                    2
                }
            }
            FixtureChannel::PanTiltSpeed(is_fine, _) => {
                if *is_fine {
                    2
                } else {
                    1
                }
            }
            FixtureChannel::Maintenance(_, _, _) => 1,
            FixtureChannel::ToggleFlags(_, _) => 1,
            FixtureChannel::NoFunction => 1,
        }
    }

    pub fn type_id(&self) -> u16 {
        match self {
            FixtureChannel::Intensity(_, _) => FIXTURE_CHANNEL_INTENSITY_ID,
            FixtureChannel::Shutter(_) => FIXTURE_CHANNEL_SHUTTER_ID,
            FixtureChannel::Zoom(_, _) => FIXTURE_CHANNEL_ZOOM_ID,
            FixtureChannel::ColorRGB(_, _)
            | FixtureChannel::ColorRGBW(_, _)
            | FixtureChannel::ColorMacro(_, _) => FIXTURE_CHANNEL_COLOR_ID,
            FixtureChannel::PositionPanTilt(_, _) => FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
            FixtureChannel::PanTiltSpeed(_, _) => FIXTURE_CHANNEL_PAN_TILT_SPEED_ID,
            FixtureChannel::Maintenance(_, id, _) => *id,
            FixtureChannel::ToggleFlags(_, _) => FIXTURE_CHANNEL_TOGGLE_FLAGS,
            FixtureChannel::NoFunction => FIXTURE_CHANNEL_NO_FUNCTION_ID,
        }
    }

    pub fn color_mode(&self) -> Result<FixtureColorChannelMode, FixtureChannelError> {
        match self {
            FixtureChannel::ColorRGB(_, _) => Ok(FixtureColorChannelMode::Rgbw),
            FixtureChannel::ColorRGBW(_, _) => Ok(FixtureColorChannelMode::Rgbw),
            FixtureChannel::ColorMacro(_, _) => Ok(FixtureColorChannelMode::Macro),
            _ => Err(FixtureChannelError::WrongFixtureChannelType),
        }
    }

    pub fn color_macro_map(&self) -> Result<&HashMap<u8, [f32; 4]>, FixtureChannelError> {
        match self {
            FixtureChannel::ColorMacro(map, _) => Ok(map),
            _ => Err(FixtureChannelError::WrongFixtureChannelType),
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
            FIXTURE_CHANNEL_SHUTTER_ID => "Shutter".to_owned(),
            FIXTURE_CHANNEL_ZOOM_ID => "Zoom".to_owned(),
            FIXTURE_CHANNEL_COLOR_ID => "Color".to_owned(),
            FIXTURE_CHANNEL_POSITION_PAN_TILT_ID => "PositionPanTilt".to_owned(),
            FIXTURE_CHANNEL_PAN_TILT_SPEED_ID => "PanTiltSpeed".to_owned(),
            FIXTURE_CHANNEL_TOGGLE_FLAGS => "ToggleFlags".to_owned(),
            FIXTURE_CHANNEL_NO_FUNCTION_ID => "NoFunction".to_owned(),
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
        updatable_handler: &UpdatableHandler,
        grand_master: f32,
    ) -> Result<Vec<u8>, FixtureChannelError> {
        let fixture_id = fixture.id();

        match self {
            FixtureChannel::Intensity(is_fine, _) => {
                let channel_value = fixture
                    .channel_value(
                        FIXTURE_CHANNEL_INTENSITY_ID,
                        preset_handler,
                        updatable_handler,
                    )
                    .map_err(FixtureChannelError::FixtureError)?;

                let (intens_coarse, intens_fine) = Self::float_to_coarse_and_fine(
                    channel_value.as_single(
                        preset_handler,
                        fixture.id(),
                        FIXTURE_CHANNEL_INTENSITY_ID,
                    )? * grand_master,
                );

                if *is_fine {
                    Ok(vec![intens_coarse, intens_fine])
                } else {
                    Ok(vec![intens_coarse])
                }
            }
            FixtureChannel::Shutter(_) => {
                let channel_value = fixture
                    .channel_value(
                        FIXTURE_CHANNEL_SHUTTER_ID,
                        preset_handler,
                        updatable_handler,
                    )
                    .map_err(FixtureChannelError::FixtureError)?;

                Ok(vec![
                    (channel_value.as_single(
                        preset_handler,
                        fixture_id,
                        FIXTURE_CHANNEL_SHUTTER_ID,
                    )? * 255.0) as u8,
                ])
            }
            FixtureChannel::Zoom(is_fine, _) => {
                let channel_value = fixture
                    .channel_value(FIXTURE_CHANNEL_ZOOM_ID, preset_handler, updatable_handler)
                    .map_err(FixtureChannelError::FixtureError)?;

                let (zoom_coarse, zoom_fine) = Self::float_to_coarse_and_fine(
                    channel_value.as_single(preset_handler, fixture_id, FIXTURE_CHANNEL_ZOOM_ID)?,
                );

                if *is_fine {
                    Ok(vec![zoom_coarse, zoom_fine])
                } else {
                    Ok(vec![zoom_coarse])
                }
            }
            FixtureChannel::ColorRGB(is_fine, _) => {
                let channel_value = fixture
                    .channel_value(FIXTURE_CHANNEL_COLOR_ID, preset_handler, updatable_handler)
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
                    .channel_value(FIXTURE_CHANNEL_COLOR_ID, preset_handler, updatable_handler)
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
            FixtureChannel::ColorMacro(_, _) => {
                let channel_value = fixture
                    .channel_value(FIXTURE_CHANNEL_COLOR_ID, preset_handler, updatable_handler)
                    .map_err(FixtureChannelError::FixtureError)?;

                let value = channel_value.as_single(
                    preset_handler,
                    fixture_id,
                    FIXTURE_CHANNEL_COLOR_ID,
                )?;

                let (value, _) = Self::float_to_coarse_and_fine(value);

                Ok(vec![value])
            }
            FixtureChannel::PositionPanTilt(is_fine, _) => {
                let channel_value = fixture
                    .channel_value(
                        FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
                        preset_handler,
                        updatable_handler,
                    )
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
            FixtureChannel::PanTiltSpeed(is_fine, _) => {
                let channel_value = fixture
                    .channel_value(
                        FIXTURE_CHANNEL_PAN_TILT_SPEED_ID,
                        preset_handler,
                        updatable_handler,
                    )
                    .map_err(FixtureChannelError::FixtureError)?;

                let pan_tilt_speed = channel_value.as_single(
                    preset_handler,
                    fixture_id,
                    FIXTURE_CHANNEL_PAN_TILT_SPEED_ID,
                )?;

                let (value_coarse, value_fine) = Self::float_to_coarse_and_fine(pan_tilt_speed);

                if *is_fine {
                    Ok(vec![value_coarse, value_fine])
                } else {
                    Ok(vec![value_coarse])
                }
            }
            FixtureChannel::Maintenance(_, id, _) => {
                let channel_value = fixture
                    .channel_value(*id, preset_handler, updatable_handler)
                    .map_err(FixtureChannelError::FixtureError)?;

                Ok(vec![
                    (channel_value.as_single(preset_handler, fixture_id, *id)? * 255.0) as u8,
                ])
            }
            FixtureChannel::ToggleFlags(flags, _) => {
                let channel_value = fixture
                    .channel_value(
                        FIXTURE_CHANNEL_TOGGLE_FLAGS,
                        preset_handler,
                        updatable_handler,
                    )
                    .map_err(FixtureChannelError::FixtureError)?;

                let flag_name = channel_value.as_toggle_flag(fixture_id)?;

                let value: u8 = flag_name.map(|f| *flags.get(&f).unwrap_or(&0)).unwrap_or(0);

                Ok(vec![value])
            }
            FixtureChannel::NoFunction => Ok(vec![0]),
        }
    }
}
