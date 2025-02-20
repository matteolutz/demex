use super::PRESET_GRID_ELEMENT_SIZE;

pub fn preset_grid_row_ui<R>(
    ui: &mut egui::Ui,
    name: &str,
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

            // draw the name of the row header to the center of the allocated painter
            painter.text(
                response.rect.center(),
                egui::Align2::CENTER_CENTER,
                name,
                egui::FontId::default(),
                egui::Color32::WHITE,
            );

            egui::ScrollArea::horizontal().show(ui, add_contents)
        })
    })
    .inner
}
