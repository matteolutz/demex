pub fn get_upper_7_bit(val: u16) -> u8 {
    (val >> 8) as u8 & 0x7F
}

pub fn get_lower_7_bit(val: u16) -> u8 {
    (val & 0xFF) as u8 & 0x7F
}
