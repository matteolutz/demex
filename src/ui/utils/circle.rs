pub fn point_lies_in_radius(origin: egui::Pos2, radius: f32, point: egui::Pos2) -> bool {
    (point.x - origin.x).powf(2.0) + (point.y - origin.y).powf(2.0) < radius.powf(2.0)
}
