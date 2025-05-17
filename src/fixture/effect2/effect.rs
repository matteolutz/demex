use std::f32;

use serde::{Deserialize, Serialize};

use super::wave::Effect2Wave;

pub type AttributeList = Vec<String>;

fn f32_one() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub struct Effect2Part {
    wave: Effect2Wave,
    attributes: Vec<String>,

    /// Phase offfset (in deg)
    #[serde(default = "f32_one")]
    phase_offset: f32,

    /// number of phases
    #[serde(default = "f32_one")]
    phase_multiplier: f32,
}

impl Default for Effect2Part {
    fn default() -> Self {
        Self {
            wave: Effect2Wave::default(),
            attributes: Vec::default(),
            phase_offset: 1.0,
            phase_multiplier: 1.0,
        }
    }
}

impl Effect2Part {
    pub fn wave(&self) -> &Effect2Wave {
        &self.wave
    }

    pub fn wave_mut(&mut self) -> &mut Effect2Wave {
        &mut self.wave
    }

    pub fn attributes(&self) -> &[String] {
        &self.attributes
    }

    pub fn attributes_mut(&mut self) -> &mut Vec<String> {
        &mut self.attributes
    }

    pub fn phase_offset_mut(&mut self) -> &mut f32 {
        &mut self.phase_offset
    }

    pub fn phase_multiplier_mut(&mut self) -> &mut f32 {
        &mut self.phase_multiplier
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub struct Effect2 {
    parts: Vec<Effect2Part>,
}

impl Effect2 {
    pub fn parts(&self) -> &[Effect2Part] {
        &self.parts
    }

    pub fn parts_mut(&mut self) -> &mut Vec<Effect2Part> {
        &mut self.parts
    }

    pub fn attributes(&self) -> impl Iterator<Item = &String> {
        self.parts.iter().flat_map(|part| &part.attributes)
    }

    pub fn attribute_value(
        &self,
        attribute_name: &str,
        time: f64,
        phase_offset_deg: f32,
        speed: f32,
    ) -> Option<f32> {
        let time_adjusted = (time as f32 * speed) - phase_offset_deg.to_radians();

        self.parts
            .iter()
            .find(|part| {
                part.attributes
                    .iter()
                    .any(|attribute| attribute == attribute_name)
            })
            .map(|part| {
                part.wave
                    .value((time_adjusted - part.phase_offset.to_radians()) / part.phase_multiplier)
            })
    }
}
