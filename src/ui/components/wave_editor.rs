use std::hash::Hash;

use crate::fixture::effect2::wave::{
    segment::{WaveSegment, WaveSegmentTouchResult},
    wave_type::WaveType,
    Effect2Wave,
};

const SNAP_DISTANCE: f32 = 15.0;

fn project_wave_point(rect: &egui::Rect, point: emath::Pos2) -> emath::Pos2 {
    rect.left_bottom() + (point.to_vec2() * emath::vec2(rect.width(), -rect.height()))
}

fn unproject_wave_point(rect: &egui::Rect, point: emath::Pos2) -> emath::Pos2 {
    let x = (point.x - rect.left()) / rect.width();
    let y = 1.0 - ((point.y - rect.top()) / rect.height());
    emath::pos2(x, y)
}

#[derive(Clone, Default)]
pub struct WaveEditorState {
    selected_point: Option<(usize, WaveSegmentTouchResult)>,
    debug_val: f32,
}

pub struct WaveEditor<'a> {
    id: egui::Id,
    wave: &'a mut Effect2Wave,
}

impl<'a> WaveEditor<'a> {
    pub fn new(id_source: impl Hash, wave: &'a mut Effect2Wave) -> Self {
        Self {
            id: egui::Id::new(id_source),
            wave,
        }
    }

    pub fn show(&'a mut self, ui: &mut egui::Ui) {
        let mut state = ui
            .ctx()
            .data_mut(|d| d.get_temp::<WaveEditorState>(self.id))
            .unwrap_or_default();

        let (response, painter) =
            ui.allocate_painter(emath::vec2(3.0, 1.0) * 300.0, egui::Sense::click_and_drag());

        let grid_rect = response.rect.shrink(30.0);

        let project_point = |point: emath::Pos2| project_wave_point(&grid_rect, point);
        let unproject_point = |point: emath::Pos2| unproject_wave_point(&grid_rect, point);

        painter.rect_stroke(
            grid_rect,
            egui::CornerRadius::same(5),
            (1.0, ecolor::Color32::WHITE),
            egui::StrokeKind::Middle,
        );

        let num_horizontal_lines = 6;
        let horizontal_offset = grid_rect.height() / (num_horizontal_lines + 1) as f32;

        for i in 1..=num_horizontal_lines {
            let from = grid_rect.left_top() + emath::vec2(0.0, i as f32 * horizontal_offset);
            let to = from + emath::vec2(grid_rect.width(), 0.0);
            painter.line(
                vec![from, to],
                egui::epaint::PathStroke::new(1.0, ecolor::Color32::WHITE.gamma_multiply(0.5)),
            );

            // TODO: draw labels
            painter.text(
                from - egui::vec2(5.0, 0.0),
                egui::Align2::RIGHT_CENTER,
                format!(
                    "{}%",
                    (((num_horizontal_lines - i + 1) as f32 / (num_horizontal_lines + 1) as f32)
                        * 100.0)
                        .round()
                ),
                egui::FontId::proportional(12.0),
                egui::Color32::WHITE,
            );
        }

        let num_vertical_lines = 11;
        let vertical_offset = grid_rect.width() / (num_vertical_lines + 1) as f32;

        for i in 1..=num_vertical_lines {
            let from = grid_rect.left_top() + emath::vec2(i as f32 * vertical_offset, 0.0);
            let to = from + emath::vec2(0.0, grid_rect.height());
            painter.line(
                vec![from, to],
                egui::epaint::PathStroke::new(1.0, ecolor::Color32::WHITE.gamma_multiply(0.5)),
            );
        }

        let num_phase_labels = 4;
        for i in 0..=num_phase_labels {
            let pos = grid_rect.left_bottom()
                + emath::vec2(
                    (i as f32 / num_phase_labels as f32) * grid_rect.width(),
                    0.0,
                );

            let deg = (i as f32 / num_phase_labels as f32) * 360.0;

            painter.text(
                pos + egui::vec2(0.0, 5.0),
                egui::Align2::CENTER_TOP,
                format!("{}°", deg),
                egui::FontId::proportional(12.0),
                egui::Color32::WHITE,
            );
        }

        for seg in self.wave.segments().iter() {
            if self.wave.wave_type() == WaveType::Bezier {
                for point in seg.control_points() {
                    painter.circle_filled(
                        project_wave_point(&grid_rect, *point),
                        3.0,
                        ecolor::Color32::RED,
                    );

                    painter.line(
                        vec![
                            project_wave_point(&grid_rect, seg.start_pos()),
                            project_wave_point(&grid_rect, *point),
                        ],
                        (1.0, ecolor::Color32::YELLOW),
                    );
                }
            }

            painter.circle_filled(
                project_wave_point(&grid_rect, seg.start_pos()),
                5.0,
                ecolor::Color32::BLUE,
            );

            /*
            if state
                .selected_point
                .is_some_and(|selected_point_idx| idx == selected_point_idx)
            {
                painter.circle_stroke(
                    grid_rect.left_bottom()
                        + (control_point.vec()
                            * emath::vec2(grid_rect.width(), -grid_rect.height())),
                    8.0,
                    (2.0, ecolor::Color32::GREEN),
                );
            }
            */
        }

        if let Some(first) = self
            .wave
            .segments()
            .iter()
            .min_by(|a, b| a.start_pos().x.partial_cmp(&b.start_pos().x).unwrap())
        {
            painter.line(
                vec![
                    project_wave_point(&grid_rect, emath::pos2(0.0, 0.0)),
                    project_wave_point(&grid_rect, emath::pos2(first.start_pos().x, 0.0)),
                    project_wave_point(&grid_rect, first.start_pos()),
                ],
                (2.0, ecolor::Color32::BLUE),
            );
        }

        if self.wave.segments().is_empty() {
            painter.line(
                vec![grid_rect.left_bottom(), grid_rect.right_bottom()],
                (2.0, ecolor::Color32::BLUE),
            );
        }

        for (a, b) in self
            .wave
            .segment_tuples(&WaveSegment::from_start_pos(emath::pos2(1.0, 0.0)))
        {
            match self.wave.wave_type() {
                WaveType::Bezier => {
                    let bezier_shape = egui::epaint::CubicBezierShape::from_points_stroke(
                        [
                            project_wave_point(&grid_rect, a.start_pos()),
                            project_wave_point(&grid_rect, a.control_points()[1]),
                            project_wave_point(&grid_rect, b.control_points()[0]),
                            project_wave_point(&grid_rect, b.start_pos()),
                        ],
                        false,
                        egui::Color32::TRANSPARENT,
                        (2.0, egui::Color32::BLUE),
                    );

                    painter.add(bezier_shape);
                }
                WaveType::Square => {
                    let x_half = emath::vec2((b.start_pos().x - a.start_pos().x) / 2.0, 0.0);
                    painter.line(
                        vec![
                            project_wave_point(&grid_rect, a.start_pos()),
                            project_wave_point(&grid_rect, a.start_pos() + x_half),
                            project_wave_point(&grid_rect, b.start_pos() - x_half),
                            project_wave_point(&grid_rect, b.start_pos()),
                        ],
                        (2.0, egui::Color32::BLUE),
                    );
                }
                WaveType::Triangle => {
                    painter.line(
                        vec![
                            project_wave_point(&grid_rect, a.start_pos()),
                            project_wave_point(&grid_rect, b.start_pos()),
                        ],
                        (2.0, egui::Color32::BLUE),
                    );
                }
            }
        }

        ui.vertical(|ui| {
            ui.add(egui::Slider::new(&mut state.debug_val, 0.0..=1.0));
            let wave_value = self
                .wave
                .value(state.debug_val * 2.0 * std::f32::consts::PI);

            painter.circle_filled(
                project_wave_point(&grid_rect, egui::pos2(state.debug_val, wave_value)),
                10.0,
                ecolor::Color32::GREEN,
            );

            egui_probe::Probe::new(self.wave.wave_type_mut())
                .with_header("Wave type")
                .show(ui);
        });

        if response.double_clicked() {
            state.selected_point = None;
            let interact_pos = response.interact_pointer_pos().unwrap();
            let pos = unproject_point(interact_pos);
            self.wave.insert_segment(pos);
        }

        if response.dragged() {
            let mut interact_pos = response.interact_pointer_pos().unwrap();

            let nearest_x = grid_rect.left()
                + ((interact_pos.x - grid_rect.left())
                    / (grid_rect.width() / (num_vertical_lines + 1) as f32))
                    .round()
                    * (grid_rect.width() / (num_vertical_lines + 1) as f32);
            let nearest_y = grid_rect.top()
                + ((interact_pos.y - grid_rect.top())
                    / (grid_rect.height() / (num_horizontal_lines + 1) as f32))
                    .round()
                    * (grid_rect.height() / (num_horizontal_lines + 1) as f32);

            let nearest_pos = emath::pos2(nearest_x, nearest_y);

            if interact_pos.distance(nearest_pos) < SNAP_DISTANCE {
                interact_pos = nearest_pos;
            }

            if let Some((seg_idx, touch_mode)) = state.selected_point {
                if let Some(segment) = self.wave.segments_mut().get_mut(seg_idx) {
                    segment.apply_dragging(touch_mode, unproject_point(interact_pos));
                }
            }
        } else if response.is_pointer_button_down_on() {
            let interact_pos = response.interact_pointer_pos().unwrap();
            state.selected_point =
                self.wave
                    .segments()
                    .iter()
                    .enumerate()
                    .find_map(|(idx, seg)| {
                        seg.touch(interact_pos, 20.0, project_point)
                            .map(|res| (idx, res))
                    });
        }

        /*
        if state.selected_point.is_some()
            && ui
                .input_mut(|reader| reader.consume_key(egui::Modifiers::NONE, egui::Key::Backspace))
        {
            self.wave
                .control_points_mut()
                .remove(state.selected_point.unwrap());
            state.selected_point = None;
        }
        */

        ui.ctx().data_mut(|d| d.insert_temp(self.id, state));
    }
}
