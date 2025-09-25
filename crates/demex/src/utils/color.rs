pub fn hsl_to_rgb([h, s, l]: [f32; 3]) -> [f32; 3] {
    if s == 0.0 {
        return [l, l, l];
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };

    let p = 2.0 * l - q;
    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    [r, g, b]
}

pub fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }

    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 0.5 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }

    p
}

/// Percentages of RGB values in the white leds.
const RGB_RATIOS: [f32; 3] = [1.0, 1.0, 1.0];

pub fn rgbw_to_rgb(mut rgbw: [f32; 4]) -> [f32; 3] {
    let max_rgb = rgbw[0].max(rgbw[1]).max(rgbw[2]);
    let white = rgbw[3];

    if white > 0.0 && max_rgb > 0.0 {
        for i in 0..3 {
            rgbw[i] *= 1.0 - white * RGB_RATIOS[i];
        }
    }

    for i in 0..3 {
        rgbw[i] += white * RGB_RATIOS[i];
    }

    [
        rgbw[0].clamp(0.0, 1.0),
        rgbw[1].clamp(0.0, 1.0),
        rgbw[2].clamp(0.0, 1.0),
    ]
}
