use gpui::{App, Bounds, Entity, Pixels, Point, Size, point, px, size};

#[derive(Debug, Clone)]
pub struct LayoutProjection {
    zoom: f32,
    center: Point<Pixels>,
    bounds: Entity<Bounds<Pixels>>,
}

impl LayoutProjection {
    pub fn new(bounds: Entity<Bounds<Pixels>>) -> Self {
        Self {
            zoom: 1.0,
            center: Point::default(),
            bounds,
        }
    }

    pub fn reset(&mut self) {
        self.zoom = 1.0;
        self.center = Point::default();
    }

    pub fn with_zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom;
        self
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn center(&self) -> &Point<Pixels> {
        &self.center
    }

    pub fn zoom_mut(&mut self) -> &mut f32 {
        &mut self.zoom
    }

    pub fn center_mut(&mut self) -> &mut Point<Pixels> {
        &mut self.center
    }
}

impl LayoutProjection {
    /// Project a given world position onto a position on the screen.
    pub fn project(&self, world_pos: Point<Pixels>, cx: &App) -> Point<Pixels> {
        let screen = self.bounds.read(cx);

        let world_pos_vec = world_pos * self.zoom;
        let screen_center = screen.center();
        let screen_pos = (self.center * self.zoom) + world_pos_vec + screen_center;
        screen_pos
    }

    /// Unproject a given screen position onto a world position.
    pub fn unproject(&self, screen_pos: Point<Pixels>, cx: &App) -> Point<Pixels> {
        let screen = self.bounds.read(cx);

        let screen_center = screen.center();
        let mut offset = screen_pos - screen_center;

        offset = offset / self.zoom;

        offset
    }

    pub fn unproject_box(&self, _screen_box: &emath::Rect, _screen: &emath::Rect) -> emath::Rect {
        todo!()
    }

    pub fn scale(&self, scale: f32) -> f32 {
        scale * self.zoom
    }

    pub fn scale_point(&self, point: Point<Pixels>) -> Point<Pixels> {
        point * self.zoom
    }

    pub fn scale_size(&self, size_to_scale: Size<Pixels>) -> Size<Pixels> {
        size(
            size_to_scale.width * self.zoom,
            size_to_scale.height * self.zoom,
        )
    }
}

pub trait PosExt {
    fn to_gpui_point(self) -> Point<Pixels>;
}
impl PosExt for emath::Pos2 {
    fn to_gpui_point(self) -> Point<Pixels> {
        point(px(self.x), px(self.y))
    }
}

/*
pub fn draw_center_of_mass(
    painter: &emath::Painter,
    center: emath::Pos2,
    radius: f32,
    color: ecolor::Color32,
    stroke_width: f32,
) {
    let rect = emath::Rect::from_center_size(center, emath::vec2(radius, radius));
    let stroke = emath::Stroke::new(stroke_width, color);

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

    let triangle1_center = center + emath::Vec2::new(0.0, -triangle_height / 2.0);
    let triangle2_center = center
        + emath::Vec2::new(
            triangle_size * (3.0_f32).sqrt() / 4.0,
            triangle_height / 4.0,
        );
    let triangle3_center = center
        + emath::Vec2::new(
            -triangle_size * (3.0_f32).sqrt() / 4.0,
            triangle_height / 4.0,
        );

    let triangle1 = [
        triangle1_center + emath::Vec2::new(-triangle_size / 2.0, triangle_height / 2.0),
        triangle1_center + emath::Vec2::new(triangle_size / 2.0, triangle_height / 2.0),
        triangle1_center + emath::Vec2::new(0.0, -triangle_height / 2.0),
    ];

    let triangle2 = [
        triangle2_center + emath::Vec2::new(-triangle_size / 2.0, -triangle_height / 2.0),
        triangle2_center + emath::Vec2::new(triangle_size / 2.0, -triangle_height / 2.0),
        triangle2_center + emath::Vec2::new(0.0, triangle_height / 2.0),
    ];

    let triangle3 = [
        triangle3_center + emath::Vec2::new(-triangle_size / 2.0, -triangle_height / 2.0),
        triangle3_center + emath::Vec2::new(triangle_size / 2.0, -triangle_height / 2.0),
        triangle3_center + emath::Vec2::new(0.0, triangle_height / 2.0),
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
*/
