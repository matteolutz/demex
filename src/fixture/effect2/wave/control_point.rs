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

    pub fn default_with(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            wave_type: Effect2WaveControlPointWaveType::default(),
        }
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn x_mut(&mut self) -> &mut f32 {
        &mut self.x
    }

    pub fn y_mut(&mut self) -> &mut f32 {
        &mut self.y
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn pos(&self) -> egui::Pos2 {
        egui::pos2(self.x, self.y)
    }

    pub fn vec(&self) -> egui::Vec2 {
        egui::vec2(self.x, self.y)
    }

    pub fn wave_type(&self) -> &Effect2WaveControlPointWaveType {
        &self.wave_type
    }

    pub fn draw_from_prev_point(
        &self,
        prev_point: &Effect2WaveControlPoint,
        painter: &egui::Painter,
        color: egui::Color32,
        project_point: impl Fn(egui::Pos2) -> egui::Pos2,
    ) {
        match self.wave_type {
            Effect2WaveControlPointWaveType::Linear => {
                painter.line(
                    vec![project_point(prev_point.pos()), project_point(self.pos())],
                    egui::epaint::PathStroke::new(2.0, color),
                );
            }
        }
    }

    pub fn value_from_prev_point(&self, prev: Option<&Self>, t: f32) -> f32 {
        if let Some(prev) = prev {
            match self.wave_type {
                Effect2WaveControlPointWaveType::Linear => {
                    let from = prev.y;
                    let to = self.y;

                    (to - from) * t + from
                }
            }
        } else if t == 1.0 {
            self.y
        } else {
            0.0
        }
    }
}
