use strum::IntoEnumIterator;

use crate::{
    color::gel::ColorGelType,
    ui::{
        components::tab_viewer::TabViewer,
        utils::{color::color_to_luma, painter::painter_layout_centered},
    },
};

pub struct GelPickerComponent {
    id_source: egui::Id,
}

impl GelPickerComponent {
    pub fn new(id_source: egui::Id) -> Self {
        Self { id_source }
    }

    pub fn show(&self, ui: &mut egui::Ui) -> Option<egui::Color32> {
        let mut selected_color = None;

        let selected_gel_type_viewer =
            TabViewer::new(self.id_source, ColorGelType::iter().collect::<Vec<_>>(), 0).show(ui);

        ui.add_space(10.0);

        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                ui.add_space(10.0);

                for gel in selected_gel_type_viewer.selected_tab.gels() {
                    let (response, painter) = ui.allocate_painter(
                        egui::vec2(ui.available_width(), 50.0),
                        egui::Sense::click(),
                    );

                    let button_rect = response.rect.shrink2(egui::vec2(10.0, 2.5));

                    painter.rect_filled(button_rect, 5.0, gel.ecolor());
                    painter_layout_centered(
                        &painter,
                        gel.name().to_owned(),
                        egui::FontId::proportional(12.0),
                        if color_to_luma(&gel.ecolor()) > 0.5 {
                            egui::Color32::BLACK
                        } else {
                            egui::Color32::WHITE
                        },
                        button_rect,
                    );

                    if response.hovered() {
                        ui.output_mut(|o| {
                            o.cursor_icon = egui::CursorIcon::PointingHand;
                        });

                        painter.rect_stroke(
                            button_rect,
                            5.0,
                            (1.0, egui::Color32::WHITE),
                            egui::StrokeKind::Middle,
                        );
                    }

                    if response.clicked() {
                        selected_color = Some(gel.ecolor());
                    }
                }

                ui.add_space(10.0);
            });

        selected_color
    }
}
