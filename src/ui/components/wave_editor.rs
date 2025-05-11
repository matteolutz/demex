use crate::fixture::effect2::wave::Effect2Wave;

pub fn wave_editor_ui(ui: &mut egui::Ui, wave: &mut Effect2Wave) {
    let (response, painter) =
        ui.allocate_painter(egui::vec2(600.0, 200.0), egui::Sense::click_and_drag());

    painter.rect_stroke(
        response.rect,
        egui::CornerRadius::same(5),
        (1.0, egui::Color32::WHITE),
        egui::StrokeKind::Middle,
    );

    let num_horizontal_lines = 4;
    let horizontal_offset = response.rect.height() / (num_horizontal_lines + 1) as f32;

    for i in 1..=num_horizontal_lines {
        let from = response.rect.left_top() + egui::vec2(0.0, i as f32 * horizontal_offset);
        let to = from + egui::vec2(response.rect.width(), 0.0);
        painter.line(
            vec![from, to],
            egui::epaint::PathStroke::new(1.0, egui::Color32::WHITE.gamma_multiply(0.5)),
        );
    }

    let num_vertical_lines = 6;
    let vertical_offset = response.rect.width() / (num_vertical_lines + 1) as f32;

    for i in 1..=num_vertical_lines {
        let from = response.rect.left_top() + egui::vec2(i as f32 * vertical_offset, 0.0);
        let to = from + egui::vec2(0.0, response.rect.height());
        painter.line(
            vec![from, to],
            egui::epaint::PathStroke::new(1.0, egui::Color32::WHITE.gamma_multiply(0.5)),
        );
    }
}
