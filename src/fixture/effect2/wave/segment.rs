use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub struct WaveSegment {
    start_pos: emath::Pos2,
    control_points: [egui::Pos2; 2],
}

impl WaveSegment {
    pub fn new(start_pos: emath::Pos2, control_points: [egui::Pos2; 2]) -> Self {
        Self {
            start_pos,
            control_points,
        }
    }

    pub fn from_start_pos(start_pos: emath::Pos2) -> Self {
        Self {
            start_pos,
            control_points: [start_pos, start_pos],
        }
    }

    pub fn start_pos(&self) -> emath::Pos2 {
        self.start_pos
    }

    pub fn control_points(&self) -> &[egui::Pos2; 2] {
        &self.control_points
    }
}
