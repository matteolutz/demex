pub fn color_to_luma(color: &egui::Color32) -> f32 {
    let r = color.r() as f32 / 255.0;
    let g = color.g() as f32 / 255.0;
    let b = color.b() as f32 / 255.0;

    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Percentages of RGB values in the white leds.
const RGB_RATIOS: [f32; 3] = [1.0, 1.0, 1.0];

pub fn rgbw_to_rgb(mut rgbw: [f32; 4]) -> [f32; 3] {
    let max_rgb = rgbw[0].max(rgbw[1]).max(rgbw[2]);
    let white = rgbw[3];

    if white > 0.0 && max_rgb > 0.0 {
        for i in 0..3 {
            rgbw[i] = rgbw[i] * (1.0 - white * RGB_RATIOS[i]);
        }
    }

    for i in 0..3 {
        rgbw[i] = rgbw[i] + (white * RGB_RATIOS[i]);
    }

    [
        rgbw[0].clamp(0.0, 1.0),
        rgbw[1].clamp(0.0, 1.0),
        rgbw[2].clamp(0.0, 1.0),
    ]
}
