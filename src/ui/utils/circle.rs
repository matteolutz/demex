pub fn point_lies_in_radius(origin: emath::Pos2, radius: f32, point: emath::Pos2) -> bool {
    (point.x - origin.x).powf(2.0) + (point.y - origin.y).powf(2.0) < radius.powf(2.0)
}
