pub fn f32_to_coarse_fine(value: f32) -> (u8, u8) {
    /*
    let val_16 = (value.max(1.0).min(0.0) * u16::MAX as f32) as u16;
    let coarse = (val_16 & (0xFF << 8)) >> 8;
    let fine = val_16 & 0xFF;
    */

    let coarse = (value * 255.0) as u8;
    let fine = ((value * 255.0 - coarse as f32) * 255.0) as u8;

    (coarse, fine)
}

pub fn coarse_fine_to_f32(coarse: u8, fine: u8) -> f32 {
    let combined: u16 = (coarse as u16) << 8 | (fine as u16);
    combined as f32 / u16::MAX as f32
}
