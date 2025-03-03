use crate::{
    fixture::layout::FixtureLayoutDecoration, ui::graphics::layout_projection::LayoutProjection,
};

impl FixtureLayoutDecoration {
    pub fn get_pos(&self, projection: &LayoutProjection, rect: &egui::Rect) -> egui::Pos2 {
        let pos = match self {
            Self::Label {
                pos,
                text: _,
                font_size: _,
            } => pos,
        };
        projection.project(pos, rect)
    }

    pub fn draw(
        &self,
        projection: &LayoutProjection,
        screen: &egui::Rect,
        painter: &egui::Painter,
    ) {
        let pos = self.get_pos(projection, screen);
        match self {
            Self::Label {
                pos: _,
                text,
                font_size,
            } => {
                painter.text(
                    pos,
                    egui::Align2::CENTER_CENTER,
                    text,
                    egui::FontId::proportional(font_size * projection.zoom()),
                    egui::Color32::WHITE,
                );
            }
        }
    }
}
