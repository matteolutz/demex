#[derive(Debug, Clone, PartialEq)]
pub struct FixtureState {
    intensity: Option<u8>,
    color: Option<(u8, u8, u8)>,
}

impl FixtureState {
    pub fn from_intensity(intensity: u8) -> Self {
        Self {
            intensity: Some(intensity),
            color: None,
        }
    }

    pub fn intensity(&self) -> &Option<u8> {
        &self.intensity
    }

    pub fn color(&self) -> &Option<(u8, u8, u8)> {
        &self.color
    }

    pub fn intensity_mult(&self, mult: f32) -> Self {
        Self {
            intensity: Some((mult.clamp(0.0, 1.0) * self.intensity.unwrap_or(0) as f32) as u8),
            color: self.color,
        }
    }
}

impl Default for FixtureState {
    fn default() -> Self {
        Self {
            intensity: None,
            color: None,
        }
    }
}

impl std::fmt::Display for FixtureState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(intensity) = self.intensity {
            write!(f, "{}%", (intensity as f32 / 255.0) * 100.0)?;
        } else {
            write!(f, "-%")?;
        }

        if let Some((r, g, b)) = self.color {
            write!(f, "\n({}, {}, {})", r, g, b)?;
        }

        Ok(())
    }
}
