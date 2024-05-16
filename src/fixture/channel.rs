use rand::Rng;

use super::error::FixtureError;

pub const FIXTURE_CHANNEL_INTENSITY_ID: u16 = 0;
pub const FIXTURE_CHANNEL_COLOR_RGB_ID: u16 = 10;
pub const FIXTURE_CHANNEL_POSITION_PAN_TILT_ID: u16 = 20;
pub const FIXTURE_CHANNEL_MAINTENANCE_ID: u16 = 30;

#[derive(Debug)]
pub enum FixtureChannel {
    Intensity(bool, Option<u8>),
    ColorRGB(bool, Option<[f32; 3]>),
    PositionPanTilt(bool, Option<(u8, u8)>),
    Maintenance(String, u16, Option<u8>),
}

impl FixtureChannel {
    pub fn intensity(is_fine: bool) -> Self {
        FixtureChannel::Intensity(is_fine, None)
    }

    pub fn color_rgb(is_fine: bool) -> Self {
        FixtureChannel::ColorRGB(is_fine, None)
    }

    pub fn position_pan_tilt(is_fine: bool) -> Self {
        FixtureChannel::PositionPanTilt(is_fine, None)
    }

    pub fn maintenance(name: &str) -> Self {
        FixtureChannel::Maintenance(
            name.to_owned(),
            rand::thread_rng().gen_range(100..u16::MAX),
            None,
        )
    }
}

impl FixtureChannel {
    fn assert_data_length(data: &[u8], expected_length: usize) -> Result<(), FixtureError> {
        if data.len() != expected_length {
            return Err(FixtureError::InvalidDataLength);
        }

        Ok(())
    }

    pub fn set_from_data_slice(&mut self, data: &[u8]) -> Result<(), FixtureError> {
        match self {
            FixtureChannel::Intensity(_, intens) => {
                Self::assert_data_length(data, 1)?;
                *intens = Some(data[0]);
            }
            FixtureChannel::ColorRGB(_, rgb) => {
                Self::assert_data_length(data, 3)?;
                *rgb = Some([data[0] as f32, data[1] as f32, data[2] as f32]);
            }
            FixtureChannel::PositionPanTilt(_, pan_tilt) => {
                Self::assert_data_length(data, 2)?;
                *pan_tilt = Some((data[0], data[1]));
            }
            FixtureChannel::Maintenance(_, _, value) => {
                Self::assert_data_length(data, 1)?;
                *value = Some(data[0]);
            }
        }

        Ok(())
    }
}

impl FixtureChannel {
    pub fn address_bandwidth(&self) -> u8 {
        match self {
            FixtureChannel::Intensity(is_fine, _) => {
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
        }
    }

    pub fn type_id(&self) -> u16 {
        match self {
            FixtureChannel::Intensity(_, _) => FIXTURE_CHANNEL_INTENSITY_ID,
            FixtureChannel::ColorRGB(_, _) => FIXTURE_CHANNEL_COLOR_RGB_ID,
            FixtureChannel::PositionPanTilt(_, _) => FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
            FixtureChannel::Maintenance(_, id, _) => *id,
        }
    }

    pub fn name(&self) -> String {
        match self {
            FixtureChannel::Intensity(_, _) => "Intensity".to_owned(),
            FixtureChannel::ColorRGB(_, _) => "ColorRGB".to_owned(),
            FixtureChannel::PositionPanTilt(_, _) => "PositionPanTilt".to_owned(),
            FixtureChannel::Maintenance(name, _, _) => name.clone(),
        }
    }

    pub fn generate_data_packet(&self) -> Vec<u8> {
        match self {
            FixtureChannel::Intensity(is_fine, intens) => {
                if *is_fine {
                    vec![intens.unwrap_or(0), 0]
                } else {
                    vec![intens.unwrap_or(0)]
                }
            }
            FixtureChannel::ColorRGB(is_fine, rgb) => {
                let [f_r, f_g, f_b] = rgb.unwrap_or([0.0, 0.0, 0.0]);

                let r = (f_r * 255.0) as u8;
                let g = (f_g * 255.0) as u8;
                let b = (f_b * 255.0) as u8;

                let r_fine = ((f_r - r as f32) * 255.0) as u8;
                let g_fine = ((f_g - g as f32) * 255.0) as u8;
                let b_fine = ((f_b - b as f32) * 255.0) as u8;

                if *is_fine {
                    vec![r, r_fine, g, g_fine, b, b_fine]
                } else {
                    vec![r, g, b]
                }
            }
            FixtureChannel::PositionPanTilt(is_fine, pan_tilt) => {
                let (pan, tilt) = pan_tilt.unwrap_or((0, 0));

                if *is_fine {
                    vec![pan, 0, tilt, 0]
                } else {
                    vec![pan, tilt]
                }
            }
            FixtureChannel::Maintenance(_, _, value) => vec![value.unwrap_or(0)],
        }
    }
}
