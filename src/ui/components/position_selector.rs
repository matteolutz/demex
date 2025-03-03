pub struct PositionSelector<'a> {
    sense: eframe::egui::Sense,
    get_or_set: Box<dyn 'a + FnMut(Option<eframe::egui::Vec2>) -> Option<eframe::egui::Vec2>>,
}

impl<'a> PositionSelector<'a> {
    pub fn new(
        get_or_set: impl 'a + FnMut(Option<eframe::egui::Vec2>) -> Option<eframe::egui::Vec2>,
    ) -> Self {
        Self {
            sense: eframe::egui::Sense::click_and_drag(),
            get_or_set: Box::new(get_or_set),
        }
    }
}

impl eframe::egui::Widget for PositionSelector<'_> {
    fn ui(mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let position = ((self.get_or_set)(None)).unwrap_or(eframe::egui::vec2(0.0, 0.0));

        let rect_size = 250.0;

        // draw a rect
        let (rect, response) =
            ui.allocate_exact_size(eframe::egui::vec2(rect_size, rect_size), self.sense);

        let vis_size = 7.5;

        if response.clicked() || response.dragged() {
            let mouse_pos = response.hover_pos();
            if let Some(mouse_pos) = mouse_pos {
                (self.get_or_set)(Some(eframe::egui::vec2(
                    ((mouse_pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0),
                    ((mouse_pos.y - rect.top()) / rect.height()).clamp(0.0, 1.0),
                )));
            }
        }

        if response.hovered() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Crosshair);
        }

        ui.painter().rect_stroke(
            rect,
            5.0,
            eframe::egui::Stroke::new(1.0, eframe::egui::Color32::WHITE),
        );

        // draw crosshair
        ui.painter().line_segment(
            [
                eframe::egui::Pos2::new(rect.left(), rect.top() + rect.height() / 2.0),
                eframe::egui::Pos2::new(rect.right(), rect.top() + rect.height() / 2.0),
            ],
            eframe::egui::Stroke::new(2.0, eframe::egui::Color32::WHITE.linear_multiply(0.2)),
        );

        ui.painter().line_segment(
            [
                eframe::egui::Pos2::new(rect.left() + rect.width() / 2.0, rect.top()),
                eframe::egui::Pos2::new(rect.left() + rect.width() / 2.0, rect.bottom()),
            ],
            eframe::egui::Stroke::new(2.0, eframe::egui::Color32::WHITE.linear_multiply(0.2)),
        );

        let vis_x = rect.left() + vis_size + (position.x * (rect.width() - 2.0 * vis_size));
        let vis_y = rect.top() + vis_size + (position.y * (rect.height() - 2.0 * vis_size));

        ui.painter().circle_filled(
            eframe::egui::Pos2::new(vis_x, vis_y),
            vis_size - 2.0,
            eframe::egui::Color32::YELLOW,
        );

        ui.painter().circle_stroke(
            eframe::egui::pos2(vis_x, vis_y),
            vis_size,
            eframe::egui::Stroke::new(2.0, eframe::egui::Color32::LIGHT_YELLOW),
        );

        response
    }
}
