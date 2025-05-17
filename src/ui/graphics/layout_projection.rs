#[derive(Debug, Copy, Clone)]
pub struct LayoutProjection {
    zoom: f32,
    center: egui::Vec2,
}

impl Default for LayoutProjection {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            center: egui::Vec2::default(),
        }
    }
}

impl LayoutProjection {
    pub fn reset(&mut self) {
        self.zoom = 1.0;
        self.center = egui::Vec2::default();
    }

    pub fn with_zoom(zoom: f32) -> Self {
        Self {
            zoom,
            center: egui::Vec2::default(),
        }
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn center(&self) -> &egui::Vec2 {
        &self.center
    }

    pub fn zoom_mut(&mut self) -> &mut f32 {
        &mut self.zoom
    }

    pub fn center_mut(&mut self) -> &mut egui::Vec2 {
        &mut self.center
    }
}

impl LayoutProjection {
    /// Project a given world position onto a position on the screen.
    pub fn project(&self, world_pos: &egui::Pos2, screen: &egui::Rect) -> egui::Pos2 {
        let world_pos_vec = world_pos.to_vec2() * self.zoom;
        let screen_center = screen.min.to_vec2() + (screen.size() / 2.0);
        let screen_pos = (self.center * self.zoom) + world_pos_vec + screen_center;
        screen_pos.to_pos2()
    }

    /// Unproject a given screen position onto a world position.
    pub fn unproject(&self, screen_pos: &egui::Pos2, screen: &egui::Rect) -> egui::Pos2 {
        let world_pos_vec = screen_pos.to_vec2();
        let screen_center = screen.min.to_vec2() + (screen.size() / 2.0);
        let mut offset = world_pos_vec - screen_center;

        offset /= self.zoom;

        offset.to_pos2()
    }

    pub fn unproject_box(&self, screen_box: &egui::Rect, screen: &egui::Rect) -> egui::Rect {
        let min = self.unproject(&screen_box.min, screen);
        let max = self.unproject(&screen_box.max, screen);
        egui::Rect::from_min_max(min, max)
    }

    pub fn scale(&self, vec: &egui::Vec2) -> egui::Vec2 {
        *vec * self.zoom
    }

    pub fn scale_ref(&self, vec: &mut egui::Vec2) {
        vec.x *= self.zoom;
        vec.y *= self.zoom;
    }
}

#[cfg(test)]
mod tests {
    use egui::Rect;

    use super::LayoutProjection;

    #[test]
    pub fn test_projection_basic() {
        let projection = LayoutProjection::with_zoom(2.0);
        let screen = Rect::from_min_max(egui::pos2(10.0, 10.0), egui::pos2(20.0, 20.0));

        let orig_point = egui::pos2(-1.0, 0.0);

        let screen_point = projection.project(&orig_point, &screen);
        assert_eq!(screen_point, egui::pos2(13.0, 15.0));

        let unprojected_point = projection.unproject(&screen_point, &screen);
        assert_eq!(unprojected_point, orig_point);
    }
}

pub fn draw_center_of_mass(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    color: ecolor::Color32,
    stroke_width: f32,
) {
    let rect = egui::Rect::from_center_size(center, egui::vec2(radius, radius));
    let stroke = egui::Stroke::new(stroke_width, color);

    painter.circle(center, radius, ecolor::Color32::TRANSPARENT, stroke);

    painter.line_segment([rect.center_top(), rect.center_bottom()], stroke);
    painter.line_segment([rect.left_center(), rect.right_center()], stroke);

    /*
    // Draw the outer circle
    painter.add(egui::epaint::Shape::Circle(egui::epaint::CircleShape {
        center,
        radius,
        fill: ecolor::Color32::TRANSPARENT,
        stroke: egui::Stroke::new(stroke_width, color),
    }));

    // Define the vertices of the three triangles
    let triangle_size = radius / 2.0;
    let triangle_height = triangle_size * (3.0_f32).sqrt() / 2.0; // Height of an equilateral triangle

    let triangle1_center = center + egui::Vec2::new(0.0, -triangle_height / 2.0);
    let triangle2_center = center
        + egui::Vec2::new(
            triangle_size * (3.0_f32).sqrt() / 4.0,
            triangle_height / 4.0,
        );
    let triangle3_center = center
        + egui::Vec2::new(
            -triangle_size * (3.0_f32).sqrt() / 4.0,
            triangle_height / 4.0,
        );

    let triangle1 = [
        triangle1_center + egui::Vec2::new(-triangle_size / 2.0, triangle_height / 2.0),
        triangle1_center + egui::Vec2::new(triangle_size / 2.0, triangle_height / 2.0),
        triangle1_center + egui::Vec2::new(0.0, -triangle_height / 2.0),
    ];

    let triangle2 = [
        triangle2_center + egui::Vec2::new(-triangle_size / 2.0, -triangle_height / 2.0),
        triangle2_center + egui::Vec2::new(triangle_size / 2.0, -triangle_height / 2.0),
        triangle2_center + egui::Vec2::new(0.0, triangle_height / 2.0),
    ];

    let triangle3 = [
        triangle3_center + egui::Vec2::new(-triangle_size / 2.0, -triangle_height / 2.0),
        triangle3_center + egui::Vec2::new(triangle_size / 2.0, -triangle_height / 2.0),
        triangle3_center + egui::Vec2::new(0.0, triangle_height / 2.0),
    ];

    // Draw the triangles
    painter.add(egui::epaint::Shape::Path(egui::epaint::PathShape {
        points: triangle1.to_vec(),
        closed: true,
        fill: color,
        stroke: egui::epaint::PathStroke::NONE,
    }));
    painter.add(egui::epaint::Shape::Path(egui::epaint::PathShape {
        points: triangle2.to_vec(),
        closed: true,
        fill: color,
        stroke: egui::epaint::PathStroke::NONE,
    }));
    painter.add(egui::epaint::Shape::Path(egui::epaint::PathShape {
        points: triangle3.to_vec(),
        closed: true,
        fill: color,
        stroke: egui::epaint::PathStroke::NONE,
    }));
    */
}
