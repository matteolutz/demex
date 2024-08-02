use std::collections::HashMap;

use crate::utils::hash;

pub const FIXTURE_CHANNEL_INTENSITY_ID: u16 = 0;
pub const FIXTURE_CHANNEL_STROBE: u16 = 1;
pub const FIXTURE_CHANNEL_ZOOM: u16 = 2;
pub const FIXTURE_CHANNEL_COLOR_RGB_ID: u16 = 10;
pub const FIXTURE_CHANNEL_POSITION_PAN_TILT_ID: u16 = 20;
pub const FIXTURE_CHANNEL_TOGGLE_FLAGS: u16 = 30;

#[derive(Debug)]
pub enum FixtureChannel {
    Intensity(bool, Option<f32>),
    Strobe(Option<f32>),
    Zoom(bool, Option<f32>),
    ColorRGB(bool, Option<[f32; 3]>),
    PositionPanTilt(bool, Option<[f32; 2]>),
    Maintenance(String, u16, Option<u8>),
    ToggleFlags(HashMap<String, u8>, Option<String>),
}

impl FixtureChannel {
    pub fn intensity(is_fine: bool) -> Self {
        FixtureChannel::Intensity(is_fine, None)
    }

    pub fn strobe() -> Self {
        FixtureChannel::Strobe(None)
    }

    pub fn zoom(is_fine: bool) -> Self {
        FixtureChannel::Zoom(is_fine, None)
    }

    pub fn color_rgb(is_fine: bool) -> Self {
        FixtureChannel::ColorRGB(is_fine, None)
    }

    pub fn position_pan_tilt(is_fine: bool) -> Self {
        FixtureChannel::PositionPanTilt(is_fine, None)
    }

    pub fn maintenance(name: &str) -> Self {
        FixtureChannel::Maintenance(name.to_owned(), hash::hash(name) as u16, None)
    }

    pub fn toggle_flags(flags: HashMap<String, u8>) -> Self {
        FixtureChannel::ToggleFlags(flags, None)
    }
}

impl FixtureChannel {
    pub fn home(&mut self) {
        match self {
            FixtureChannel::Intensity(_, intens) => *intens = None,
            FixtureChannel::Strobe(strobe) => *strobe = None,
            FixtureChannel::Zoom(_, zoom) => *zoom = None,
            FixtureChannel::ColorRGB(_, rgb) => *rgb = None,
            FixtureChannel::PositionPanTilt(_, pan_tilt) => *pan_tilt = None,
            FixtureChannel::Maintenance(_, _, value) => *value = None,
            FixtureChannel::ToggleFlags(_, value) => *value = None,
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
            FixtureChannel::ColorRGB(_, _) => FIXTURE_CHANNEL_COLOR_RGB_ID,
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
            FIXTURE_CHANNEL_COLOR_RGB_ID => "ColorRGB".to_owned(),
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

    pub fn generate_data_packet(&self) -> Vec<u8> {
        match self {
            FixtureChannel::Intensity(is_fine, intens) => {
                let (intens_coarse, intens_fine) =
                    Self::float_to_coarse_and_fine(intens.unwrap_or(0.0));

                if *is_fine {
                    vec![intens_coarse, intens_fine]
                } else {
                    vec![intens_coarse]
                }
            }
            FixtureChannel::Strobe(strobe) => vec![(strobe.unwrap_or(0.0) * 255.0) as u8],
            FixtureChannel::Zoom(is_fine, zoom) => {
                let (zoom_coarse, zoom_fine) = Self::float_to_coarse_and_fine(zoom.unwrap_or(0.0));

                if *is_fine {
                    vec![zoom_coarse, zoom_fine]
                } else {
                    vec![zoom_coarse]
                }
            }
            FixtureChannel::ColorRGB(is_fine, rgb) => {
                let [f_r, f_g, f_b] = rgb.unwrap_or([0.0, 0.0, 0.0]);

                let (r, r_fine) = Self::float_to_coarse_and_fine(f_r);
                let (g, g_fine) = Self::float_to_coarse_and_fine(f_g);
                let (b, b_fine) = Self::float_to_coarse_and_fine(f_b);

                if *is_fine {
                    vec![r, r_fine, g, g_fine, b, b_fine]
                } else {
                    vec![r, g, b]
                }
            }
            FixtureChannel::PositionPanTilt(is_fine, pan_tilt) => {
                let [pan_f, tilt_f] = pan_tilt.unwrap_or([0.0, 0.0]);

                let (pan, pan_fine) = Self::float_to_coarse_and_fine(pan_f);
                let (tilt, tilt_fine) = Self::float_to_coarse_and_fine(tilt_f);

                if *is_fine {
                    vec![pan, pan_fine, tilt, tilt_fine]
                } else {
                    vec![pan, tilt]
                }
            }
            FixtureChannel::Maintenance(_, _, value) => vec![value.unwrap_or(0)],
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
