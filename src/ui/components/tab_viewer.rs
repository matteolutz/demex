use std::hash::Hash;

pub struct TabViewerResponse<T> {
    selected_tab: u32,
    inner_response: egui::InnerResponse<T>,
}

pub struct TabViewer {
    id: egui::Id,
    tabs: Vec<String>,
    initial_selected_tab: u32,
    height: f32,
}

impl TabViewer {
    pub fn new(id_source: impl Hash, tabs: Vec<String>, selected_tab: u32, height: f32) -> Self {
        if selected_tab >= tabs.len() as u32 {
            panic!("Selected tab index out of bounds");
        }

        Self {
            id: egui::Id::new(id_source),
            tabs,
            initial_selected_tab: selected_tab,
            height,
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        let mut selected_tab = ui
            .data(|reader| reader.get_temp::<u32>(self.id))
            .unwrap_or(self.initial_selected_tab);

        let available_rect = ui.available_rect_before_wrap();
        let cell_width = available_rect.width() / self.tabs.len() as f32;

        let mut cell_rect =
            egui::Rect::from_min_size(available_rect.min, egui::vec2(cell_width, self.height));

        for (i, tab) in self.tabs.iter().enumerate() {
            let response = ui.allocate_rect(cell_rect, egui::Sense::click());

            if response.clicked() {
                selected_tab = i as u32;
            }

            let is_selected = i as u32 == selected_tab;
            let text_color = if is_selected {
                egui::Color32::WHITE
            } else {
                egui::Color32::GRAY
            };

            cell_rect.min.x += cell_width;
        }

        ui.data_mut(|writer| writer.insert_temp(self.id, selected_tab));
    }
}
