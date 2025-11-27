use std::{collections::HashMap, u8};

use crate::{
    channel3::channel_value_queue::ChannelValueQueue,
    engine::component::Component,
    fixture::{error::FixtureError, handler::error::FixtureHandlerError},
    patch::Patch,
    presets::PresetHandler,
    state::fixture_state::FixtureState,
    timing::TimingHandler,
    updatables::UpdatableHandler,
    value_source::FixtureChannelValueSourceTrait,
};

impl Component for FixtureStateHandler {}

#[derive(Debug)]
pub struct FixtureStateHandler {
    fixture_states: HashMap<u32, FixtureState>,
    grand_master: u8,
}

impl Default for FixtureStateHandler {
    fn default() -> Self {
        Self {
            fixture_states: Default::default(),
            grand_master: u8::MAX,
        }
    }
}

impl FixtureStateHandler {
    pub fn new(patch: &Patch) -> Result<Self, FixtureHandlerError> {
        // TODO: find a new place for this
        /*
        // check if the fixtures overlap
        let mut fixture_addresses: HashMap<u16, BTreeSet<u16>> = HashMap::new();

        for f in patch.fixtures() {
            let fixture_type = patch
                .fixture_type(f.fixture_type_id)
                .expect("Fixture type not found");
            let dmx_mode = fixture_type
                .dmx_mode(&f.fixture_type_dmx_mode)
                .expect("Fixture type DMX mode not found");

            let start_address = f.start_address();

            let address_footprint = (dmx_mode
                .dmx_channels
                .iter()
                .flat_map(|dmx_channel| &dmx_channel.offset)
                .flatten()
                .max()
                .copied()
                .ok_or(FixtureHandlerError::FixtureError(
                    FixtureError::GdtfMaxDmxOffsetNotFound,
                ))?) as u16;

            let end_address = start_address + address_footprint - 1;
            let address_set = fixture_addresses.entry(f.universe()).or_default();

            for i in start_address..=end_address {
                if address_set.contains(&i) {
                    return Err(FixtureHandlerError::FixtureAddressOverlap(
                        f.universe(),
                        start_address,
                        end_address,
                    ));
                }

                address_set.insert(i);
            }
        }
        */

        Ok(Self {
            fixture_states: patch
                .fixtures()
                .map(|f| (f.id, FixtureState::new(f, patch)))
                .collect::<HashMap<_, _>>(),
            grand_master: u8::MAX,
        })
    }

    pub fn grand_master(&self) -> u8 {
        self.grand_master
    }

    pub fn grand_master_mut(&mut self) -> &mut u8 {
        &mut self.grand_master
    }

    pub fn update_output_values(
        &mut self,
        patch: &Patch,
        preset_handler: &PresetHandler,
        updatable_handler: &UpdatableHandler,
        timing_handler: &TimingHandler,
    ) -> Result<(), FixtureError> {
        for (id, state) in self.fixture_states.iter_mut() {
            let (_, dmx_mode, fixture) = patch.fixture_type_and_dmx_mode_by_id(*id)?;

            for dmx_channel in &dmx_mode.dmx_channels {
                let new_output_value = state.sources().get_channel_value(
                    patch,
                    fixture,
                    state,
                    dmx_channel,
                    updatable_handler,
                    preset_handler,
                    timing_handler,
                )?;

                let output_value = state
                    .cached_output_mut()
                    .get_mut(dmx_channel.name().as_ref())
                    .unwrap();

                if output_value.value == new_output_value {
                    continue;
                }

                output_value.update(new_output_value);
            }
        }

        Ok(())
    }

    pub fn submit_output_values(
        &mut self,
        value_queue: &mut ChannelValueQueue,
        patch: &Patch,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> Result<(), FixtureError> {
        for (id, state) in self.fixture_states.iter_mut() {
            let mut updated_values = HashMap::new();
            let fixture = patch.fixture(*id)?;

            for (channel, output_value) in state.cached_output_mut() {
                if !output_value.should_output(preset_handler) {
                    continue;
                }

                let discrete_value = output_value.value().clone().to_discrete(
                    patch,
                    fixture,
                    &channel,
                    preset_handler,
                    timing_handler,
                );
                updated_values.insert(channel.clone(), discrete_value);
                output_value.reset();
            }

            value_queue.enqueue(*id, updated_values);
        }

        Ok(())
    }
}
