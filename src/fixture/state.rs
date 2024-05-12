#[derive(Debug, Clone, PartialEq)]
pub struct FixtureState {
    intensity: u8,
    color: (u8, u8, u8),
    pan: u8,
    tilt: u8,
}

impl FixtureState {
    pub fn from_intensity(intensity: u8) -> Self {
        Self {
            intensity,
            color: (0, 0, 0),
            pan: 0,
            tilt: 0,
        }
    }

    pub fn intensity(&self) -> u8 {
        self.intensity
    }

    pub fn color(&self) -> (u8, u8, u8) {
        self.color
    }

    pub fn pan(&self) -> u8 {
        self.pan
    }

    pub fn tilt(&self) -> u8 {
        self.tilt
    }

    pub fn intensity_mult(&self, mult: f32) -> Self {
        Self {
            intensity: (mult.clamp(0.0, 1.0) * self.intensity as f32) as u8,
            color: self.color,
            pan: self.pan,
            tilt: self.tilt,
        }
    }
}

impl Default for FixtureState {
    fn default() -> Self {
        Self {
            intensity: 0,
            color: (0, 0, 0),
            pan: 0,
            tilt: 0,
        }
    }
}

impl std::fmt::Display for FixtureState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}%", (self.intensity as f32 / 255.0) * 100.0)?;

        let (r, g, b) = self.color;
        write!(f, "\n({}, {}, {})", r, g, b)?;

        Ok(())
    }
}
