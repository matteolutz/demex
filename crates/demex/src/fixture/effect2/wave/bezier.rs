pub fn cubic_bezier(points: [emath::Pos2; 4], t: f32) -> emath::Pos2 {
    let h = 1.0 - t;
    let a = t * t * t;
    let b = 3.0 * t * t * h;
    let c = 3.0 * t * h * h;
    let d = h * h * h;
    let result = points[3].to_vec2() * a
        + points[2].to_vec2() * b
        + points[1].to_vec2() * c
        + points[0].to_vec2() * d;
    result.to_pos2()
}
