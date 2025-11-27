use std::collections::HashMap;

use crate::{
    channel3::{
        channel_value_discrete::FixtureChannelDiscreteValue, channel_value_queue::ChannelValueQueue,
    },
    patch::Patch,
};

pub struct DmxResolver {
    universe_data: HashMap<u16, [u8; 512]>,
    output_values: HashMap<u32, HashMap<String, FixtureChannelDiscreteValue>>,
}

impl DmxResolver {
    pub fn resovle(&mut self, value_queue: &mut ChannelValueQueue, patch: &Patch) {
        for entry in value_queue.inner_mut().drain(..) {
            let fixture_patch = patch.fixture(entry.fixture_id).unwrap();
            let fixture_output_values = self.output_values.entry(entry.fixture_id).or_default();

            let mut dynamic_data = HashMap::new();

            for (channel, value) in entry.values.into_iter() {
                // update local output values
                fixture_output_values.insert(channel.clone(), value.clone());

                let (dmx_channel, _) = fixture_patch.get_channel(patch, &channel).unwrap();
                let Some(channel_offsets) = &dmx_channel.offset else {
                    continue;
                };

                // generate dmx value
                let Some(dmx_value) = value.to_dmx(
                    patch,
                    fixture_patch,
                    fixture_output_values,
                    dmx_channel,
                    &mut dynamic_data,
                    1.0,
                ) else {
                    continue;
                };

                let mut real_dmx_value = dmx_value.to(channel_offsets.len() as u8);
                for offset in channel_offsets.iter().rev() {
                    let universe_offset = (fixture_patch.start_address - 1) + (*offset as u16 - 1);

                    self.universe_data
                        .entry(fixture_patch.universe)
                        .or_insert_with(|| [0; 512])[universe_offset as usize] =
                        (real_dmx_value & 0xFF) as u8;
                    real_dmx_value >>= 8;
                }
            }
        }
    }
}
