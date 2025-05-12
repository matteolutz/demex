use std::hash::Hash;

use itertools::Itertools;

use crate::{
    fixture::effect2::wave::{control_point::Effect2WaveControlPoint, Effect2Wave},
    ui::utils::circle::point_lies_in_radius,
};

#[derive(Clone, Default)]
pub struct WaveEditorState {
    selected_point: Option<usize>,
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
            ui.allocate_painter(egui::vec2(3.0, 1.0) * 300.0, egui::Sense::click_and_drag());

        let grid_rect = response.rect.shrink(20.0);

        painter.rect_stroke(
            grid_rect,
            egui::CornerRadius::same(5),
            (1.0, egui::Color32::WHITE),
            egui::StrokeKind::Middle,
        );

        let num_horizontal_lines = 4;
        let horizontal_offset = grid_rect.height() / (num_horizontal_lines + 1) as f32;

        for i in 1..=num_horizontal_lines {
            let from = grid_rect.left_top() + egui::vec2(0.0, i as f32 * horizontal_offset);
            let to = from + egui::vec2(grid_rect.width(), 0.0);
            painter.line(
                vec![from, to],
                egui::epaint::PathStroke::new(1.0, egui::Color32::WHITE.gamma_multiply(0.5)),
            );
        }

        let num_vertical_lines = 9;
        let vertical_offset = grid_rect.width() / (num_vertical_lines + 1) as f32;

        for i in 1..=num_vertical_lines {
            let from = grid_rect.left_top() + egui::vec2(i as f32 * vertical_offset, 0.0);
            let to = from + egui::vec2(0.0, grid_rect.height());
            painter.line(
                vec![from, to],
                egui::epaint::PathStroke::new(1.0, egui::Color32::WHITE.gamma_multiply(0.5)),
            );
        }

        for (idx, control_point) in self.wave.control_points().iter().enumerate() {
            painter.circle_filled(
                grid_rect.left_bottom()
                    + (control_point.vec() * egui::vec2(grid_rect.width(), -grid_rect.height())),
                5.0,
                egui::Color32::BLUE,
            );

            if state
                .selected_point
                .is_some_and(|selected_point_idx| idx == selected_point_idx)
            {
                painter.circle_stroke(
                    grid_rect.left_bottom()
                        + (control_point.vec()
                            * egui::vec2(grid_rect.width(), -grid_rect.height())),
                    8.0,
                    (2.0, egui::Color32::GREEN),
                );
            }
        }

        if let Some(first) = self
            .wave
            .control_points()
            .iter()
            .min_by(|a, b| a.x().partial_cmp(&b.x()).unwrap())
        {
            painter.line(
                vec![
                    grid_rect.left_bottom(),
                    grid_rect.left_bottom()
                        + (first.vec() * egui::vec2(grid_rect.width(), -grid_rect.height())),
                ],
                (2.0, egui::Color32::BLUE),
            );
        }

        if let Some(last) = self
            .wave
            .control_points()
            .iter()
            .max_by(|a, b| a.x().partial_cmp(&b.x()).unwrap())
        {
            painter.line(
                vec![
                    grid_rect.left_bottom()
                        + (last.vec() * egui::vec2(grid_rect.width(), -grid_rect.height())),
                    grid_rect.right_bottom(),
                ],
                (2.0, egui::Color32::BLUE),
            );
        }

        if self.wave.control_points().is_empty() {
            painter.line(
                vec![grid_rect.left_bottom(), grid_rect.right_bottom()],
                (2.0, egui::Color32::BLUE),
            );
        }

        for (a, b) in self
            .wave
            .control_points()
            .iter()
            .sorted_by(|a, b| a.x().partial_cmp(&b.x()).unwrap())
            .tuple_windows::<(_, _)>()
        {
            b.draw_from_prev_point(a, &painter, egui::Color32::BLUE, |point| {
                grid_rect.left_bottom()
                    + (point.to_vec2() * egui::vec2(grid_rect.width(), -grid_rect.height()))
            });
        }

        if response.double_clicked() {
            state.selected_point = None;
            let interact_pos = response.interact_pointer_pos().unwrap();
            let x = (interact_pos.x - grid_rect.left()) / grid_rect.width();
            let y = 1.0 - ((interact_pos.y - grid_rect.top()) / grid_rect.height());
            self.wave
                .control_points_mut()
                .push(Effect2WaveControlPoint::default_with(x, y));
        }

        if response.clicked() {
            let interact_pos = response.interact_pointer_pos().unwrap();
            state.selected_point = self.wave.control_points_mut().iter_mut().position(|point| {
                point_lies_in_radius(
                    interact_pos,
                    20.0,
                    grid_rect.left_bottom()
                        + (point.vec() * egui::vec2(grid_rect.width(), -grid_rect.height())),
                )
            });
        }

        if response.dragged() {
            let interact_pos = response.interact_pointer_pos().unwrap();

            if let Some(selected_point_idx) = state.selected_point {
                let selected_point = &mut self.wave.control_points_mut()[selected_point_idx];
                let x = (interact_pos.x - grid_rect.left()) / grid_rect.width();
                let y = 1.0 - ((interact_pos.y - grid_rect.top()) / grid_rect.height());
                *selected_point.x_mut() = x.clamp(0.0, 1.0);
                *selected_point.y_mut() = y.clamp(0.0, 1.0);
            } else {
                state.selected_point =
                    self.wave.control_points_mut().iter_mut().position(|point| {
                        point_lies_in_radius(
                            interact_pos,
                            20.0,
                            grid_rect.left_bottom()
                                + (point.vec()
                                    * egui::vec2(grid_rect.width(), -grid_rect.height())),
                        )
                    });
            }
        }

        if state.selected_point.is_some()
            && ui
                .input_mut(|reader| reader.consume_key(egui::Modifiers::NONE, egui::Key::Backspace))
        {
            self.wave
                .control_points_mut()
                .remove(state.selected_point.unwrap());
            state.selected_point = None;
        }

        ui.ctx().data_mut(|d| d.insert_temp(self.id, state));
    }
}
