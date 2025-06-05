use crate::ui::utils::{color::color_to_luma, painter::painter_layout_centered};

use super::PRESET_GRID_ELEMENT_SIZE;

pub fn preset_grid_row_ui<R>(
    ui: &mut egui::Ui,
    name: &str,
    id: Option<u32>,
    header_color: ecolor::Color32,
    add_contents: impl FnOnce(&mut egui::Ui, usize, f32) -> R,
) -> egui::InnerResponse<R> {
    let available_width = ui.available_width();

    ui.push_id(name, |ui| {
        egui::Grid::new(name)
            .spacing(egui::vec2(1.0, 0.0))
            .show(ui, |ui| {
                let (response, painter) =
                    ui.allocate_painter(PRESET_GRID_ELEMENT_SIZE.into(), egui::Sense::empty());
                painter.rect_filled(
                    response.rect,
                    egui::CornerRadius {
                        nw: 5,
                        sw: 5,
                        ne: 0,
                        se: 0,
                    },
                    header_color,
                );

                if let Some(id) = id {
                    painter.text(
                        response.rect.left_top() + (2.0, 5.0).into(),
                        egui::Align2::LEFT_TOP,
                        id,
                        egui::FontId::proportional(12.0),
                        ecolor::Color32::WHITE,
                    );
                }

                // draw the name of the row header to the center of the allocated painter
                painter_layout_centered(
                    &painter,
                    name.to_owned(),
                    egui::FontId::default(),
                    if color_to_luma(&header_color) > 0.5 {
                        ecolor::Color32::BLACK
                    } else {
                        ecolor::Color32::WHITE
                    },
                    response.rect,
                );

                let max_buttons_f = (available_width / PRESET_GRID_ELEMENT_SIZE[0]) - 1.0;
                let max_buttons = max_buttons_f.floor() as usize;

                let per_button_overhead = ((max_buttons_f - max_buttons as f32)
                    / max_buttons as f32)
                    * PRESET_GRID_ELEMENT_SIZE[0]
                    - 2.0;

                add_contents(ui, max_buttons, per_button_overhead)
            })
            .inner
    })
}
