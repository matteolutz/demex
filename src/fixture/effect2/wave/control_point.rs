use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EguiProbe, Default)]
pub enum Effect2WaveControlPointWaveType {
    #[default]
    Linear,
}

#[derive(Debug, Clone, Serialize, Deserialize, EguiProbe, Default)]
pub struct Effect2WaveControlPoint {
    x: f32,
    y: f32,
    wave_type: Effect2WaveControlPointWaveType,
}

impl Effect2WaveControlPoint {
    pub fn new(x: f32, y: f32, wave_type: Effect2WaveControlPointWaveType) -> Self {
        Self { x, y, wave_type }
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn wave_type(&self) -> &Effect2WaveControlPointWaveType {
        &self.wave_type
    }

    pub fn value_to_prev_point(&self, prev: Option<&Self>, t: f32) -> f32 {
        if let Some(prev) = prev {
            match self.wave_type {
                Effect2WaveControlPointWaveType::Linear => {
                    let from = prev.y;
                    let to = self.y;

                    (to - from) * t + from
                }
            }
        } else {
            if t == 1.0 {
                self.y
            } else {
                0.0
            }
        }
    }
}
