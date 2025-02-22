use std::{f32, time, u16};

pub fn approx_equal(a: f32, b: f32, decimal_places: u8) -> bool {
    let factor = 10.0f32.powi(decimal_places as i32);
    let a = (a * factor).trunc();
    let b = (b * factor).trunc();
    a == b
}

pub fn approx_equals_color(
    [a_r, a_g, a_b]: [f32; 3],
    [b_r, b_g, b_b]: [f32; 3],
    decimal_places: u8,
) -> bool {
    approx_equal(a_r, b_r, decimal_places)
        && approx_equal(a_g, b_g, decimal_places)
        && approx_equal(a_b, b_b, decimal_places)
}

pub fn f32_to_coarse(value: f32) -> u8 {
    (value * 255.0) as u8
}

pub fn f32_to_coarse_fine(value: f32) -> (u8, u8) {
    let combined_u16 = (value * u16::MAX as f32) as u16;

    // shift to the right, so we get the upper byte
    let coarse: u8 = (combined_u16 >> 8) as u8;

    // mask out the lower byte
    let fine: u8 = (combined_u16 & 0xFF) as u8;

    (coarse, fine)
}

pub fn coarse_to_f32(coarse: u8) -> f32 {
    coarse as f32 / 255.0
}

pub fn coarse_fine_to_f32(coarse: u8, fine: u8) -> f32 {
    let combined: u16 = (coarse as u16) << 8 | (fine as u16);
    combined as f32 / u16::MAX as f32
}

pub fn zero_one_sin(x: f32) -> f32 {
    (f32::sin(x - f32::consts::FRAC_PI_2) + 1.0) / 2.0
}

pub fn zero_one_sin_snap_in_start(x: f32) -> f32 {
    let phase = x % (2.0 * f32::consts::PI);
    // when we're in the first half of the sine wave, we're at 1.0
    // -> snap in
    if phase < f32::consts::PI {
        1.0
    // when we're in the second half of the sine wave, we're at 0.0
    } else {
        zero_one_sin(x)
    }
}

pub fn zero_one_sin_snap_in_end(x: f32) -> f32 {
    let phase = x % (2.0 * f32::consts::PI);
    if phase < f32::consts::PI {
        0.0
    } else {
        zero_one_sin(x)
    }
}

pub fn zero_one_sin_snap_out_start(x: f32) -> f32 {
    let phase = x % (2.0 * f32::consts::PI);

    if phase > f32::consts::PI {
        0.0
    } else {
        zero_one_sin(x)
    }
}

pub fn zero_one_sin_snap_out_end(x: f32) -> f32 {
    let phase = x % (2.0 * f32::consts::PI);

    if phase > f32::consts::PI {
        1.0
    } else {
        zero_one_sin(x)
    }
}

pub fn snap_both_start(x: f32) -> f32 {
    let phase = x % (2.0 * f32::consts::PI);

    if phase < f32::consts::PI {
        1.0
    } else {
        0.0
    }
}

pub fn instant_diff_secs(later: time::Instant, earlier: time::Instant) -> f64 {
    let diff = later.checked_duration_since(earlier);
    if let Some(diff) = diff {
        diff.as_secs_f64()
    } else {
        -earlier.duration_since(later).as_secs_f64()
    }
}
