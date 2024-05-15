#[derive(Debug)]
pub enum FixtureChannel {
    Intensity(Option<u8>),
    ColorRGB(Option<(u8, u8, u8)>),
    PositionPanTilt(Option<(u8, u8)>),
    Maintenance(String, Option<u8>),
}

impl FixtureChannel {
    pub fn intensity() -> Self {
        FixtureChannel::Intensity(None)
    }

    pub fn color_rgb() -> Self {
        FixtureChannel::ColorRGB(None)
    }

    pub fn position_pan_tilt() -> Self {
        FixtureChannel::PositionPanTilt(None)
    }

    pub fn maintenance(name: &str) -> Self {
        FixtureChannel::Maintenance(name.to_owned(), None)
    }
}

impl FixtureChannel {
    pub fn address_bandwidth(&self) -> u8 {
        match self {
            FixtureChannel::Intensity(_) => 1,
            FixtureChannel::ColorRGB(_) => 3,
            FixtureChannel::PositionPanTilt(_) => 2,
            FixtureChannel::Maintenance(_, _) => 1,
        }
    }

    pub fn name(&self) -> String {
        match self {
            FixtureChannel::Intensity(_) => "Intensity".to_string(),
            FixtureChannel::ColorRGB(_) => "Color RGB".to_string(),
            FixtureChannel::PositionPanTilt(_) => "Position Pan/Tilt".to_string(),
            FixtureChannel::Maintenance(name, _) => name.to_string(),
        }
    }

    pub fn generate_data_packet(&self) -> Vec<u8> {
        match self {
            FixtureChannel::Intensity(intens) => vec![intens.unwrap_or(0)],
            FixtureChannel::ColorRGB(rgb) => {
                let (r, g, b) = rgb.unwrap_or((0, 0, 0));
                vec![r, g, b]
            }
            FixtureChannel::PositionPanTilt(pan_tilt) => {
                let (pan, tilt) = pan_tilt.unwrap_or((0, 0));
                vec![pan, tilt]
            }
            FixtureChannel::Maintenance(_, value) => vec![value.unwrap_or(0)],
        }
    }
}
