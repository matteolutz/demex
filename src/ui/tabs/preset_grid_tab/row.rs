use crate::ui::utils::{color::color_to_luma, painter::painter_layout_centered};

use super::PRESET_GRID_ELEMENT_SIZE;

pub fn preset_grid_row_ui<R>(
    ui: &mut egui::Ui,
    name: &str,
    id: Option<u32>,
    header_color: egui::Color32,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<egui::scroll_area::ScrollAreaOutput<R>> {
    ui.push_id(name, |ui| {
        ui.horizontal(|ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(1.0, 0.0);

            let (response, painter) = ui.allocate_painter(
                PRESET_GRID_ELEMENT_SIZE.into(),
                egui::Sense {
                    click: false,
                    drag: false,
                    focusable: false,
                },
            );
            painter.rect_filled(
                response.rect,
                egui::Rounding {
                    nw: 5.0,
                    sw: 5.0,
                    ne: 0.0,
                    se: 0.0,
                },
                header_color,
            );

            if let Some(id) = id {
                painter.text(
                    response.rect.left_top() + (2.0, 5.0).into(),
                    egui::Align2::LEFT_TOP,
                    id,
                    egui::FontId::proportional(12.0),
                    egui::Color32::WHITE,
                );
            }

            // draw the name of the row header to the center of the allocated painter
            painter_layout_centered(
                &painter,
                name.to_owned(),
                egui::FontId::default(),
                if color_to_luma(&header_color) > 0.5 {
                    egui::Color32::BLACK
                } else {
                    egui::Color32::WHITE
                },
                response.rect,
            );

            egui::ScrollArea::horizontal().show(ui, add_contents)
        })
    })
    .inner
}
