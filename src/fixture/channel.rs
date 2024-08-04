use std::collections::HashMap;

use color::FixtureColorValue;
use position::FixturePositionValue;

use crate::utils::hash;

use super::presets::PresetHandler;

pub mod color;
pub mod position;

pub const FIXTURE_CHANNEL_INTENSITY_ID: u16 = 0;
pub const FIXTURE_CHANNEL_STROBE: u16 = 1;
pub const FIXTURE_CHANNEL_ZOOM: u16 = 2;
pub const FIXTURE_CHANNEL_COLOR_ID: u16 = 10;
pub const FIXTURE_CHANNEL_POSITION_PAN_TILT_ID: u16 = 20;
pub const FIXTURE_CHANNEL_TOGGLE_FLAGS: u16 = 30;

pub type FixtureId = u32;

#[derive(Debug, Clone, PartialEq)]
pub enum FixtureChannel {
    Intensity(bool, f32),
    Strobe(f32),
    Zoom(bool, f32),
    ColorRGB(bool, FixtureColorValue),
    PositionPanTilt(bool, FixturePositionValue),
    Maintenance(String, u16, u8),
    ToggleFlags(HashMap<String, u8>, Option<String>),
}

impl FixtureChannel {
    pub fn intensity(is_fine: bool) -> Self {
        FixtureChannel::Intensity(is_fine, 0.0)
    }

    pub fn strobe() -> Self {
        FixtureChannel::Strobe(0.0)
    }

    pub fn zoom(is_fine: bool) -> Self {
        FixtureChannel::Zoom(is_fine, 0.0)
    }

    pub fn color_rgb(is_fine: bool) -> Self {
        FixtureChannel::ColorRGB(is_fine, FixtureColorValue::Rgbw([0.0, 0.0, 0.0, 0.0]))
    }

    pub fn position_pan_tilt(is_fine: bool) -> Self {
        FixtureChannel::PositionPanTilt(is_fine, FixturePositionValue::PanTilt([0.0, 0.0]))
    }

    pub fn maintenance(name: &str) -> Self {
        FixtureChannel::Maintenance(name.to_owned(), hash::hash(name) as u16, 0)
    }

    pub fn toggle_flags(flags: HashMap<String, u8>) -> Self {
        FixtureChannel::ToggleFlags(flags, None)
    }
}

impl FixtureChannel {
    pub fn home(&mut self) {
        match self {
            FixtureChannel::Intensity(_, intens) => *intens = 0.0,
            FixtureChannel::Strobe(strobe) => *strobe = 0.0,
            FixtureChannel::Zoom(_, zoom) => *zoom = 0.0,
            FixtureChannel::ColorRGB(_, rgb) => {
                *rgb = FixtureColorValue::Rgbw([0.0, 0.0, 0.0, 0.0])
            }
            FixtureChannel::PositionPanTilt(_, position) => {
                *position = FixturePositionValue::PanTilt([0.0, 0.0])
            }
            FixtureChannel::Maintenance(_, _, value) => *value = 0,
            FixtureChannel::ToggleFlags(_, value) => *value = None,
        }
    }

    pub fn is_home(&self) -> bool {
        match self {
            FixtureChannel::Intensity(_, intens) => *intens == 0.0,
            FixtureChannel::Strobe(strobe) => *strobe == 0.0,
            FixtureChannel::Zoom(_, zoom) => *zoom == 0.0,
            FixtureChannel::ColorRGB(_, color) => {
                *color == FixtureColorValue::Rgbw([0.0, 0.0, 0.0, 0.0])
            }
            FixtureChannel::PositionPanTilt(_, position) => {
                *position == FixturePositionValue::PanTilt([0.0, 0.0])
            }
            FixtureChannel::Maintenance(_, _, value) => *value == 0,
            FixtureChannel::ToggleFlags(_, value) => value.is_none(),
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

    pub fn generate_data_packet(&self, fixture_id: u32, preset_handler: &PresetHandler) -> Vec<u8> {
        match self {
            FixtureChannel::Intensity(is_fine, intens) => {
                let (intens_coarse, intens_fine) = Self::float_to_coarse_and_fine(*intens);

                if *is_fine {
                    vec![intens_coarse, intens_fine]
                } else {
                    vec![intens_coarse]
                }
            }
            FixtureChannel::Strobe(strobe) => vec![(strobe * 255.0) as u8],
            FixtureChannel::Zoom(is_fine, zoom) => {
                let (zoom_coarse, zoom_fine) = Self::float_to_coarse_and_fine(*zoom);

                if *is_fine {
                    vec![zoom_coarse, zoom_fine]
                } else {
                    vec![zoom_coarse]
                }
            }
            FixtureChannel::ColorRGB(is_fine, color) => {
                let [f_r, f_g, f_b, _] = match color {
                    FixtureColorValue::Rgbw([r, g, b, _]) => [*r, *g, *b, 0.0],
                    FixtureColorValue::Preset(preset_id) => preset_handler
                        .get_color_for_fixture(*preset_id, fixture_id)
                        .unwrap_or([0.0, 0.0, 0.0, 0.0]),
                };

                let (r, r_fine) = Self::float_to_coarse_and_fine(f_r);
                let (g, g_fine) = Self::float_to_coarse_and_fine(f_g);
                let (b, b_fine) = Self::float_to_coarse_and_fine(f_b);

                if *is_fine {
                    vec![r, r_fine, g, g_fine, b, b_fine]
                } else {
                    vec![r, g, b]
                }
            }
            FixtureChannel::PositionPanTilt(is_fine, position) => {
                let [pan_f, tilt_f] = match position {
                    FixturePositionValue::PanTilt([pan, tilt]) => [*pan, *tilt],
                    FixturePositionValue::Preset(preset_id) => preset_handler
                        .get_position_for_fixture(*preset_id, fixture_id)
                        .unwrap_or([0.0, 0.0]),
                };

                let (pan, pan_fine) = Self::float_to_coarse_and_fine(pan_f);
                let (tilt, tilt_fine) = Self::float_to_coarse_and_fine(tilt_f);

                if *is_fine {
                    vec![pan, pan_fine, tilt, tilt_fine]
                } else {
                    vec![pan, tilt]
                }
            }
            FixtureChannel::Maintenance(_, _, value) => vec![*value],
            FixtureChannel::ToggleFlags(flags, set_flag) => {
                let value: u8 = set_flag
                    .as_ref()
                    .map(|f| *flags.get(f).unwrap_or(&0))
                    .unwrap_or(0);

                vec![value]
            }
        }
    }
}
