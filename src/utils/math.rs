use std::u16;

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
    (x.sin() + 1.0) / 2.0
}
