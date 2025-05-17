use std::{fmt::Display, hash::Hash};

use crate::ui::utils::painter::painter_layout_centered;

pub struct TabViewerResponse<Tab: Display + Eq + Copy> {
    pub selected_tab: Tab,
}

pub struct TabViewer<Tab: Display + Eq + Copy> {
    id: egui::Id,
    tabs: Vec<Tab>,
    initial_selected_tab: usize,
    bypass_state: bool,
}

impl<Tab: Display + Eq + Copy> TabViewer<Tab> {
    pub fn new(id_source: impl Hash, tabs: Vec<Tab>, selected_tab: usize) -> Self {
        if selected_tab >= tabs.len() {
            panic!("Selected tab index out of bounds");
        }

        Self {
            id: egui::Id::new(id_source),
            tabs,
            initial_selected_tab: selected_tab,
            bypass_state: false,
        }
    }

    pub fn new_without_state(id_source: impl Hash, tabs: Vec<Tab>, selected_tab: usize) -> Self {
        if selected_tab >= tabs.len() {
            panic!("Selected tab index out of bounds");
        }

        Self {
            id: egui::Id::new(id_source),
            tabs,
            initial_selected_tab: selected_tab,
            bypass_state: true,
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) -> TabViewerResponse<Tab> {
        let mut selected_tab = if self.bypass_state {
            self.initial_selected_tab
        } else {
            ui.data(|reader| reader.get_temp::<usize>(self.id))
                .unwrap_or(self.initial_selected_tab)
        };

        let available_rect = ui.available_rect_before_wrap();
        let cell_width = available_rect.width() / self.tabs.len() as f32;
        let cell_height = 40.0;

        let mut cell_rect =
            egui::Rect::from_min_size(available_rect.min, egui::vec2(cell_width, cell_height));

        for (i, tab) in self.tabs.iter().enumerate() {
            let response = ui.allocate_rect(cell_rect, egui::Sense::click());

            if response.hovered() {
                ui.ctx()
                    .output_mut(|writer| writer.cursor_icon = egui::CursorIcon::PointingHand);
            }

            if response.clicked() {
                selected_tab = i;
            }

            let is_selected = i == selected_tab;
            let rect_fill = ui
                .style()
                .interact_selectable(&response, is_selected)
                .bg_fill;

            let text_color = if is_selected {
                ecolor::Color32::WHITE
            } else {
                ecolor::Color32::GRAY
            };

            ui.painter().rect_filled(cell_rect, 0.0, rect_fill);

            if is_selected {
                ui.painter().rect_stroke(
                    cell_rect,
                    0.0,
                    (2.0, ui.style().visuals.text_color()),
                    egui::StrokeKind::Middle,
                );
            }

            painter_layout_centered(
                ui.painter(),
                tab.to_string(),
                egui::FontId::proportional(12.0),
                text_color,
                cell_rect,
            );

            cell_rect.min.x += cell_width;
            cell_rect.max.x += cell_width;
        }

        if !self.bypass_state {
            ui.data_mut(|writer| writer.insert_temp(self.id, selected_tab));
        }

        TabViewerResponse {
            selected_tab: self.tabs[selected_tab],
        }
    }
}
