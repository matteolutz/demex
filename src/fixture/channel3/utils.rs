use std::num::NonZero;

pub fn max_value(bytes: NonZero<u8>) -> u64 {
    match bytes.get() {
        1 => u8::MAX as u64,
        2 => u16::MAX as u64,
        4 => u32::MAX as u64,
        8 => u64::MAX,
        _ => 0,
    }
}

pub fn multiply_dmx_value_f32(
    dmx_value: gdtf::values::DmxValue,
    mult: f32,
) -> gdtf::values::DmxValue {
    let value = (dmx_value.value() as f32 / max_value(dmx_value.bytes()) as f32)
        * mult
        * max_value(dmx_value.bytes()) as f32;
    gdtf::values::DmxValue::new(value as u64, dmx_value.bytes(), dmx_value.shifting()).unwrap()
}

pub fn multiply_dmx_value(
    dmx_value: gdtf::values::DmxValue,
    mult: gdtf::values::DmxValue,
) -> gdtf::values::DmxValue {
    let mult = mult.value() as f32 / max_value(mult.bytes()) as f32;
    multiply_dmx_value_f32(dmx_value, mult)
}

pub fn dmx_value_to_f32(dmx_value: gdtf::values::DmxValue) -> f32 {
    dmx_value.value() as f32 / max_value(dmx_value.bytes()) as f32
}

pub fn mix_dmx_value(
    dmx_a: gdtf::values::DmxValue,
    dmx_b: gdtf::values::DmxValue,
    mix: f32,
) -> gdtf::values::DmxValue {
    let a = dmx_a.value() as f32 / max_value(dmx_a.bytes()) as f32;
    let b = dmx_b.value() as f32 / max_value(dmx_b.bytes()) as f32;
    let value = (a * mix + b * (1.0 - mix)) * max_value(dmx_a.bytes()) as f32;
    gdtf::values::DmxValue::new(value as u64, dmx_a.bytes(), dmx_a.shifting()).unwrap()
}
