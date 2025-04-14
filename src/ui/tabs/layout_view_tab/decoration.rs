use crate::{
    fixture::layout::FixtureLayoutDecoration, ui::graphics::layout_projection::LayoutProjection,
};

impl FixtureLayoutDecoration {
    pub fn draw(
        &self,
        projection: &LayoutProjection,
        screen: &egui::Rect,
        painter: &egui::Painter,
    ) {
        match self {
            Self::Label {
                pos,
                text,
                font_size,
            } => {
                let projected_pos = projection.project(pos, screen);

                painter.text(
                    projected_pos,
                    egui::Align2::CENTER_CENTER,
                    text,
                    egui::FontId::proportional(font_size * projection.zoom()),
                    egui::Color32::WHITE,
                );
            }
            Self::Rect {
                min,
                max,
                stroke_width,
            } => {
                let projected_min = projection.project(min, screen);
                let projected_max = projection.project(max, screen);

                painter.rect_stroke(
                    egui::Rect::from_min_max(projected_min, projected_max),
                    egui::CornerRadius::ZERO,
                    egui::Stroke::new(stroke_width * projection.zoom(), egui::Color32::WHITE),
                    egui::StrokeKind::Middle,
                );
            }
        }
    }
}
