pub fn color_to_luma(color: &egui::Color32) -> f32 {
    let r = color.r() as f32 / 255.0;
    let g = color.g() as f32 / 255.0;
    let b = color.b() as f32 / 255.0;

    0.2126 * r + 0.7152 * g + 0.0722 * b
}
