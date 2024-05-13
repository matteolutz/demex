#[derive(Debug, Clone, PartialEq)]
pub struct FixtureState {
    intensity: u8,
    color: Option<(u8, u8, u8)>,
}

impl FixtureState {
    pub fn from_intensity(intensity: u8) -> Self {
        Self {
            intensity,
            color: None,
        }
    }

    pub fn intensity(&self) -> u8 {
        self.intensity
    }

    pub fn color(&self) -> &Option<(u8, u8, u8)> {
        &self.color
    }

    pub fn intensity_mult(&self, mult: f32) -> Self {
        Self {
            intensity: (mult.clamp(0.0, 1.0) * self.intensity as f32) as u8,
            color: self.color,
        }
    }
}

impl Default for FixtureState {
    fn default() -> Self {
        Self {
            intensity: 0,
            color: None,
        }
    }
}

impl std::fmt::Display for FixtureState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}%", (self.intensity as f32 / 255.0) * 100.0)?;

        if let Some((r, g, b)) = self.color {
            write!(f, "\n({}, {}, {})", r, g, b)?;
        }

        Ok(())
    }
}
