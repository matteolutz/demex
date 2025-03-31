#[derive(Default)]
pub struct LayoutViewInputOffset {
    pub zoom: f32,
    pub center: egui::Vec2,
}

pub fn read_layout_view_input(ui: &egui::Ui, rect: egui::Rect, zoom: f32) -> LayoutViewInputOffset {
    ui.input(|reader| {
        if let Some(multi_touch) = reader.multi_touch() {
            LayoutViewInputOffset {
                zoom: multi_touch.zoom_delta,
                center: multi_touch.translation_delta / zoom,
            }
        } else {
            if reader.pointer.hover_pos().is_none()
                || !rect.contains(reader.pointer.hover_pos().unwrap())
            {
                return LayoutViewInputOffset::default();
            }

            let mut zoom_offset: f32 = 0.0;
            let mut center_offset: egui::Vec2 = egui::Vec2::ZERO;

            let zoom_delta = reader.zoom_delta();
            if zoom_delta != 1.0 && reader.pointer.hover_pos().is_some() {
                let hover_pos = reader.pointer.hover_pos().unwrap();
                let center_delta: egui::Vec2 = (hover_pos - (rect.min + rect.size() / 2.0)) / zoom;

                let zoom_amount = zoom_delta - 1.0;

                zoom_offset += zoom_amount;
                center_offset -= center_delta * ((zoom_amount / 2.0).min(1.0));
            } else if reader.raw_scroll_delta != egui::Vec2::ZERO {
                let offset = reader.raw_scroll_delta / zoom;
                center_offset += offset;
            }

            LayoutViewInputOffset {
                zoom: zoom_offset,
                center: center_offset,
            }
        }
    })
}
