pub const FIXTURE_CHANNEL_INTENSITY_ID: u16 = 0;
pub const FIXTURE_CHANNEL_STROBE: u16 = 1;
pub const FIXTURE_CHANNEL_COLOR_RGB_ID: u16 = 10;
pub const FIXTURE_CHANNEL_POSITION_PAN_TILT_ID: u16 = 20;
pub const FIXTURE_CHANNEL_MAINTENANCE_ID: u16 = 30;

fn my_hash<T>(obj: T) -> u64
where
    T: std::hash::Hash,
{
    let mut hasher = std::hash::DefaultHasher::new();
    obj.hash(&mut hasher);
    std::hash::Hasher::finish(&hasher)
}

#[derive(Debug)]
pub enum FixtureChannel {
    Intensity(bool, Option<f32>),
    Strobe(Option<f32>),
    ColorRGB(bool, Option<[f32; 3]>),
    PositionPanTilt(bool, Option<[f32; 2]>),
    Maintenance(String, u16, Option<u8>),
}

impl FixtureChannel {
    pub fn intensity(is_fine: bool) -> Self {
        FixtureChannel::Intensity(is_fine, None)
    }

    pub fn strobe() -> Self {
        FixtureChannel::Strobe(None)
    }

    pub fn color_rgb(is_fine: bool) -> Self {
        FixtureChannel::ColorRGB(is_fine, None)
    }

    pub fn position_pan_tilt(is_fine: bool) -> Self {
        FixtureChannel::PositionPanTilt(is_fine, None)
    }

    pub fn maintenance(name: &str) -> Self {
        FixtureChannel::Maintenance(name.to_owned(), my_hash(name) as u16, None)
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
            FixtureChannel::Strobe(_) => FIXTURE_CHANNEL_STROBE,
            FixtureChannel::ColorRGB(_, _) => FIXTURE_CHANNEL_COLOR_RGB_ID,
            FixtureChannel::PositionPanTilt(_, _) => FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
            FixtureChannel::Maintenance(_, id, _) => *id,
        }
    }

    pub fn name(&self) -> String {
        Self::name_by_id(self.type_id())
    }

    pub fn name_by_id(id: u16) -> String {
        match id {
            FIXTURE_CHANNEL_INTENSITY_ID => "Intensity".to_owned(),
            FIXTURE_CHANNEL_STROBE => "Strobe".to_owned(),
            FIXTURE_CHANNEL_COLOR_RGB_ID => "ColorRGB".to_owned(),
            FIXTURE_CHANNEL_POSITION_PAN_TILT_ID => "PositionPanTilt".to_owned(),
            FIXTURE_CHANNEL_MAINTENANCE_ID => "Maintenance".to_owned(),
            _ => "Unknown".to_owned(),
        }
    }

    pub fn generate_data_packet(&self) -> Vec<u8> {
        match self {
            FixtureChannel::Intensity(is_fine, intens) => {
                if *is_fine {
                    vec![(intens.unwrap_or(0.0) * 255.0) as u8, 0]
                } else {
                    vec![(intens.unwrap_or(0.0) * 255.0) as u8]
                }
            }
            FixtureChannel::Strobe(strobe) => vec![(strobe.unwrap_or(0.0) * 255.0) as u8],
            FixtureChannel::ColorRGB(is_fine, rgb) => {
                let [f_r, f_g, f_b] = rgb.unwrap_or([0.0, 0.0, 0.0]);

                let r = (f_r * 255.0) as u8;
                let g = (f_g * 255.0) as u8;
                let b = (f_b * 255.0) as u8;

                let r_fine = ((f_r * 255.0 - r as f32) * 255.0) as u8;
                let g_fine = ((f_g * 255.0 - g as f32) * 255.0) as u8;
                let b_fine = ((f_b * 255.0 - b as f32) * 255.0) as u8;

                if *is_fine {
                    vec![r, r_fine, g, g_fine, b, b_fine]
                } else {
                    vec![r, g, b]
                }
            }
            FixtureChannel::PositionPanTilt(is_fine, pan_tilt) => {
                let [pan_f, tilt_f] = pan_tilt.unwrap_or([0.0, 0.0]);

                let pan = (pan_f * 255.0) as u8;
                let tilt = (tilt_f * 255.0) as u8;

                let pan_fine = ((pan_f * 255.0 - pan as f32) * 255.0) as u8;
                let tilt_fine = ((tilt_f * 255.0 - tilt as f32) * 255.0) as u8;

                if *is_fine {
                    vec![pan, pan_fine, tilt, tilt_fine]
                } else {
                    vec![pan, tilt]
                }
            }
            FixtureChannel::Maintenance(_, _, value) => vec![value.unwrap_or(0)],
        }
    }
}
