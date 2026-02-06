use std::{collections::HashMap, num::NonZero};

use crate::{
    channel3::{
        attribute::FixtureChannel3Attribute, channel_value_discrete::FixtureChannelDiscreteValue,
        clamped_value::ClampedValue,
    },
    fixture::Fixture,
};

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
    let value = (b * mix + a * (1.0 - mix)) * max_value(dmx_a.bytes()) as f32;
    gdtf::values::DmxValue::new(value as u64, dmx_a.bytes(), dmx_a.shifting()).unwrap()
}

pub trait HashMapExt {
    fn get_color(&self, fixture: &Fixture) -> Option<ecolor::Color32>;
}

impl HashMapExt for HashMap<FixtureChannel3Attribute, FixtureChannelDiscreteValue> {
    fn get_color(&self, fixture: &Fixture) -> Option<ecolor::Color32> {
        fn get_cf_and_value(
            this: &HashMap<FixtureChannel3Attribute, FixtureChannelDiscreteValue>,
            attribute: &FixtureChannel3Attribute,
            fixture: &Fixture,
        ) -> Option<ClampedValue> {
            this.get(attribute)
                .and_then(|value| fixture.channel_function(attribute).map(|cf| (cf, value)))
                .map(|(cf, value)| value.to_clamped(cf))
        }

        // check if we have r, g, b
        if let (Some(r), Some(g), Some(b)) = (
            get_cf_and_value(self, &FixtureChannel3Attribute::ColorAddR, fixture),
            get_cf_and_value(self, &FixtureChannel3Attribute::ColorAddG, fixture),
            get_cf_and_value(self, &FixtureChannel3Attribute::ColorAddB, fixture),
        ) {
            let w = get_cf_and_value(self, &FixtureChannel3Attribute::ColorAddW, fixture);

            return Some(ecolor::Color32::from_rgba_premultiplied(
                r.to_u8(),
                g.to_u8(),
                b.to_u8(),
                w.map_or(0, |w| w.to_u8()),
            ));
        }

        // TODO: color wheels??
        // TODO: subtractive colors

        None
    }
}
