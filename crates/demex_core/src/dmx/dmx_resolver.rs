use std::collections::HashMap;

use demex_dmx::{DemexDmxOutput, DemexDmxOutputTrait};

use crate::{
    channel3::{
        channel_value_discrete::FixtureChannelDiscreteValue,
        channel_value_queue::ChannelValueQueueEntry,
    },
    patch::Patch,
};

#[derive(Debug, Default)]
pub struct DmxResolver {
    old_universe_data: HashMap<u16, [u8; 512]>,
    pub universe_data: HashMap<u16, [u8; 512]>,

    output_values: HashMap<u32, HashMap<String, FixtureChannelDiscreteValue>>,
}

impl DmxResolver {
    pub fn send<'a>(
        &mut self,
        outputs: impl Iterator<Item = &'a mut DemexDmxOutput>,
        force: bool,
    ) -> usize {
        let universes_to_send = force.then(|| {
            self.universe_data
                .iter()
                .filter(|(universe, data)| {
                    data != &self
                        .old_universe_data
                        .get(universe)
                        .unwrap_or_else(|| &[0; 512])
                })
                .map(|(universe, _)| *universe)
                .collect::<Vec<_>>()
        });

        for output in outputs {
            let Some(output_universes) = output.config().universes() else {
                for universe in self.universe_data.keys().filter(|universe| {
                    universes_to_send
                        .as_ref()
                        .is_none_or(|universes_to_send| universes_to_send.contains(universe))
                }) {
                    let data = self
                        .universe_data
                        .get(universe)
                        .unwrap_or_else(|| &[0; 512]);
                    let _ = output.send(*universe, data).inspect_err(|err| {
                        log::warn!("Failed to send on output {:?}: {}", output.config(), err)
                    });
                }
                continue;
            };

            for universe in output_universes.iter().filter(|universe| {
                universes_to_send
                    .as_ref()
                    .is_none_or(|universes_to_send| universes_to_send.contains(universe))
            }) {
                let data = self
                    .universe_data
                    .get(universe)
                    .unwrap_or_else(|| &[0; 512]);
                let _ = output.send(*universe, data).inspect_err(|err| {
                    log::warn!("Failed to send on output {:?}: {}", output.config(), err)
                });
            }
        }

        let updated_universes = universes_to_send
            .map(|u| u.len())
            .unwrap_or(self.universe_data.len());

        self.old_universe_data = self.universe_data.clone();

        updated_universes
    }

    pub fn resovle(&mut self, values: Vec<ChannelValueQueueEntry>, patch: &Patch) {
        // update output values
        for entry in &values {
            let fixture_output_values = self.output_values.entry(entry.fixture_id).or_default();
            for (channel, value) in &entry.values {
                fixture_output_values.insert(channel.clone(), value.clone());
            }
        }

        // calculate dmx values
        for entry in values {
            let fixture_patch = patch.fixture(entry.fixture_id).unwrap();
            let fixture_output_values = self.output_values.entry(entry.fixture_id).or_default();

            let mut dynamic_data = HashMap::new();

            for (channel, value) in entry.values.into_iter() {
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
