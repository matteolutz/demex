use std::collections::HashMap;

use demex_dmx::{DemexDmxOutput, DemexDmxOutputTrait};

use crate::{
    channel3::{
        attribute::FixtureChannel3Attribute, channel_value_discrete::FixtureChannelDiscreteValue,
        channel_value_queue::ChannelValueQueueEntry,
    },
    fixture::{FixtureChannelFunctionInitial, FixtureChannelFunctionKind, FixturePath},
    patch::Patch,
};

fn compare_universe_data(a: &[u8; 512], b: &[u8; 512]) -> bool {
    a.iter().zip(b.iter()).all(|(a, b)| a == b)
}

#[derive(Debug, Default)]
pub struct DmxResolver {
    old_universe_data: HashMap<u16, [u8; 512]>,
    pub universe_data: HashMap<u16, [u8; 512]>,

    output_values:
        HashMap<FixturePath, HashMap<FixtureChannel3Attribute, FixtureChannelDiscreteValue>>,
}

impl DmxResolver {
    pub fn send<'a>(
        &mut self,
        outputs: impl Iterator<Item = &'a mut DemexDmxOutput>,
        force: bool,
    ) -> usize {
        let universes_to_send = (!force)
            .then(|| {
                self.universe_data
                    .iter()
                    .filter(|(universe, data)| {
                        self.old_universe_data
                            .get(universe)
                            .is_none_or(|old_data| !compare_universe_data(data, old_data))
                    })
                    .map(|(universe, _)| *universe)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec![]);

        for output in outputs {
            let Some(output_universes) = output.config().universes() else {
                for universe in self
                    .universe_data
                    .keys()
                    .filter(|universe| force || universes_to_send.contains(universe))
                {
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

            for universe in output_universes
                .iter()
                .filter(|universe| force || universes_to_send.contains(universe))
            {
                let data = self
                    .universe_data
                    .get(universe)
                    .unwrap_or_else(|| &[0; 512]);
                let _ = output.send(*universe, data).inspect_err(|err| {
                    log::warn!("Failed to send on output {:?}: {}", output.config(), err)
                });
            }
        }

        let updated_universes = if force {
            self.universe_data.len()
        } else {
            universes_to_send.len()
        };

        self.old_universe_data = self.universe_data.clone();

        updated_universes
    }

    fn universe(&mut self, universe: u16) -> &mut [u8; 512] {
        self.universe_data
            .entry(universe)
            .or_insert_with(|| [0; 512])
    }

    pub fn resovle(&mut self, values: Vec<ChannelValueQueueEntry>, patch: &Patch) {
        // update output values
        for entry in &values {
            let fixture_output_values = self.output_values.entry(entry.fixture_path).or_default();
            for (channel, (value, _)) in &entry.values {
                fixture_output_values.insert(channel.clone(), value.clone());
            }
        }

        // calculate dmx values
        for entry in values {
            let fixture_patch = patch.fixture(&entry.fixture_path).unwrap();
            let master_value = entry.master_value;

            for (attribute, value) in entry.sorted_values(fixture_patch) {
                let Some(mut channel_function) = fixture_patch.channel_function(&attribute) else {
                    continue;
                };

                // TODO: fix this
                // we only get changed values, so if we have two non inital channel functions
                // having a value != Home, and one of them is being homed, the initial cf will be
                // used instead of the non homed non initial cf
                if value.is_home() {
                    match channel_function.initial {
                        FixtureChannelFunctionInitial::Other(initial_attribute) => {
                            channel_function = fixture_patch
                                .channel_function(&initial_attribute)
                                .unwrap_or(channel_function);
                        }
                        _ => {}
                    }
                }

                match &channel_function.kind {
                    FixtureChannelFunctionKind::Physical { addresses } => {
                        // project the value (0.0..=1.0) into the CF range

                        let value_mult = if channel_function.should_react_to_master() {
                            master_value.as_f32()
                        } else {
                            1.0
                        };

                        let value = value.to_projected(channel_function, value_mult);

                        let mut dmx_value = value.to_bytes(addresses.len());

                        for address in addresses.iter().rev() {
                            self.universe(address.universe)[address.channel as usize - 1] =
                                (dmx_value & 0xFF) as u8; // is masking even necessary here?
                            dmx_value >>= 8;
                        }
                    }
                    FixtureChannelFunctionKind::Virtual { relations: _ } => {
                        // TODO
                    }
                }
            }
        }
    }
}
