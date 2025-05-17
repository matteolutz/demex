use crate::{
    fixture::layout::{FixtureLayoutEntry, FixtureLayoutEntryType},
    ui::{graphics::layout_projection::LayoutProjection, utils::rect::rect_vertices},
};

const DRAW_DIRECTION: bool = false;

impl FixtureLayoutEntry {
    pub fn get_pos_and_size(
        &self,
        projection: &LayoutProjection,
        rect: &egui::Rect,
    ) -> (egui::Pos2, egui::Vec2) {
        let size = projection.scale(self.size());
        let pos = projection.project(self.position(), rect);

        (pos, size)
    }

    pub fn draw(
        &self,
        projection: &LayoutProjection,
        screen: &egui::Rect,
        painter: &egui::Painter,
        fixture_color: ecolor::Color32,
        fixture_direction: Option<egui::Vec2>,
        is_selected: bool,
        is_about_to_selected: bool,
        label: impl ToString,
    ) {
        let (pos, size) = self.get_pos_and_size(projection, screen);

        let stroke_width = 0.25 * projection.zoom();
        let stroke_color = if is_selected {
            ecolor::Color32::GREEN
        } else if is_about_to_selected {
            ecolor::Color32::BLUE
        } else {
            ecolor::Color32::WHITE
        };

        match self.entry_type() {
            FixtureLayoutEntryType::Rect => {
                let top_left = pos - (size / 2.0);
                let rect = egui::Rect::from_min_size(top_left, size);

                painter.rect_stroke(
                    rect,
                    0.0,
                    (stroke_width, stroke_color),
                    egui::StrokeKind::Middle,
                );

                painter.rect_filled(
                    egui::Rect::from_min_size(
                        top_left + egui::vec2(stroke_width, stroke_width),
                        size - egui::vec2(2.0 * stroke_width, 2.0 * stroke_width),
                    ),
                    0.0,
                    fixture_color,
                );

                if let Some(fixture_direction) = fixture_direction {
                    if DRAW_DIRECTION {
                        let pos_in_rect = rect.left_top() + (fixture_direction * size);

                        for pos in rect_vertices(&rect) {
                            // draw line from pos to pos_in_rect
                            painter.line_segment(
                                [pos, pos_in_rect],
                                egui::Stroke::new(
                                    0.25 * projection.zoom(),
                                    ecolor::Color32::YELLOW.gamma_multiply(0.5),
                                ),
                            );
                        }

                        painter.circle_filled(
                            pos_in_rect,
                            0.5 * projection.zoom(),
                            ecolor::Color32::YELLOW.gamma_multiply(0.5),
                        );
                    }
                }
            }
            FixtureLayoutEntryType::Circle => {
                let radius = size.x.min(size.y) / 2.0;

                painter.circle_stroke(pos, radius, (stroke_width, stroke_color));

                painter.circle_filled(pos, radius - stroke_width, fixture_color);
            }
            FixtureLayoutEntryType::Triangle => {
                let triangle_height = size.x.min(size.y);
                let side_length = triangle_height * (2.0 / f32::sqrt(3.0));

                let points_outer = vec![
                    pos + egui::vec2(-side_length / 2.0, triangle_height / 2.0),
                    pos + egui::vec2(side_length / 2.0, triangle_height / 2.0),
                    pos + egui::vec2(0.0, -triangle_height / 2.0),
                ];

                painter.add(egui::epaint::PathShape::convex_polygon(
                    points_outer.clone(),
                    fixture_color,
                    egui::epaint::PathStroke::NONE,
                ));

                painter.add(egui::epaint::PathShape::closed_line(
                    points_outer,
                    (stroke_width, stroke_color),
                ));
            }
        }

        painter.text(
            pos + egui::vec2(0.0, size.y + 2.0),
            egui::Align2::CENTER_TOP,
            label,
            egui::FontId::proportional(2.0 * projection.zoom()),
            if is_selected {
                ecolor::Color32::GREEN
            } else {
                ecolor::Color32::WHITE
            },
        );
    }
}
