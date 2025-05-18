use serde::{Deserialize, Serialize};

use crate::ui::utils::circle::point_lies_in_radius;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum WaveSegmentTouchResult {
    StartPos,
    ControlPoint1,
    ControlPoint2,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub struct WaveSegment {
    start_pos: emath::Pos2,
    control_points: [emath::Pos2; 2],
}

impl WaveSegment {
    pub fn new(start_pos: emath::Pos2, control_points: [emath::Pos2; 2]) -> Self {
        Self {
            start_pos,
            control_points,
        }
    }

    pub fn from_start_pos(start_pos: emath::Pos2) -> Self {
        Self {
            start_pos,
            control_points: [
                start_pos + egui::vec2(-0.1, 0.0),
                start_pos + egui::vec2(0.1, 0.0),
            ],
        }
    }

    pub fn start_pos(&self) -> emath::Pos2 {
        self.start_pos
    }

    pub fn control_points(&self) -> &[emath::Pos2; 2] {
        &self.control_points
    }

    pub fn touch(
        &self,
        touch_pos: emath::Pos2,
        radius: f32,
        project_pos: impl Fn(emath::Pos2) -> emath::Pos2,
    ) -> Option<WaveSegmentTouchResult> {
        if point_lies_in_radius(touch_pos, radius, project_pos(self.start_pos)) {
            Some(WaveSegmentTouchResult::StartPos)
        } else if point_lies_in_radius(touch_pos, radius, project_pos(self.control_points[0])) {
            Some(WaveSegmentTouchResult::ControlPoint1)
        } else if point_lies_in_radius(touch_pos, radius, project_pos(self.control_points[1])) {
            Some(WaveSegmentTouchResult::ControlPoint2)
        } else {
            None
        }
    }

    pub fn apply_dragging(
        &mut self,
        touch_mode: WaveSegmentTouchResult,
        unprojected_touch_pos: emath::Pos2,
    ) {
        let unprojected_touch_pos =
            unprojected_touch_pos.clamp(emath::Pos2::ZERO, emath::pos2(1.0, 1.0));

        match touch_mode {
            WaveSegmentTouchResult::StartPos => {
                let drag_delta = unprojected_touch_pos - self.start_pos;
                self.start_pos = unprojected_touch_pos;

                self.control_points[0] += drag_delta;
                self.control_points[1] += drag_delta;
            }
            WaveSegmentTouchResult::ControlPoint1 => {
                self.control_points[0] = unprojected_touch_pos;
            }
            WaveSegmentTouchResult::ControlPoint2 => {
                self.control_points[1] = unprojected_touch_pos;
            }
        }
    }
}
