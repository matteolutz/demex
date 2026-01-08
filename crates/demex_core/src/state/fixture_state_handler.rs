use std::{collections::HashMap, sync::mpsc, u8};

use crate::{
    channel3::{
        attribute::FixtureChannel3Attribute, channel_value::FixtureChannelValue3,
        channel_value_queue::ChannelValueQueueEntry,
    },
    engine::component::Component,
    fixture::{Fixture, FixturePath, FixturePathMatchLevel, error::FixtureError},
    patch::Patch,
    presets::PresetHandler,
    selection::FixtureSelection,
    state::fixture_state::FixtureState,
    timing::TimingHandler,
    updatables::UpdatableHandler,
    value_source::FixtureChannelValueSourceTrait,
};

impl Component for FixtureStateHandler {}

#[derive(Debug)]
pub struct FixtureStateHandler {
    fixture_states: HashMap<FixturePath, FixtureState>,
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
    pub fn new<'a>(fixtures: impl IntoIterator<Item = &'a Fixture>) -> Result<Self, FixtureError> {
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
            fixture_states: fixtures
                .into_iter()
                .map(|f| (f.path(), FixtureState::new(f)))
                .collect(),
            grand_master: u8::MAX,
        })
    }

    pub fn grand_master(&self) -> u8 {
        self.grand_master
    }

    pub fn grand_master_mut(&mut self) -> &mut u8 {
        &mut self.grand_master
    }

    pub fn fixtures(&self) -> &HashMap<FixturePath, FixtureState> {
        &self.fixture_states
    }

    pub fn fixture(&self, fixture_path: &FixturePath) -> Result<&FixtureState, FixtureError> {
        self.fixture_states
            .get(fixture_path)
            .ok_or(FixtureError::NotFound(*fixture_path))
    }

    pub fn fixture_mut(
        &mut self,
        fixture_path: &FixturePath,
    ) -> Result<&mut FixtureState, FixtureError> {
        self.fixture_states
            .get_mut(fixture_path)
            .ok_or(FixtureError::NotFound(*fixture_path))
    }

    pub fn home_all(&mut self, clear_sources: bool) -> Result<(), FixtureError> {
        for (_, state) in self.fixture_states.iter_mut() {
            state.home(clear_sources)?;
        }

        Ok(())
    }

    pub fn update_output_values(
        &mut self,
        patch: &Patch,
        preset_handler: &PresetHandler,
        updatable_handler: &UpdatableHandler,
        timing_handler: &TimingHandler,
        highlight: Option<&FixtureSelection>,
        updated_output_values: &mut HashMap<
            FixturePath,
            HashMap<FixtureChannel3Attribute, FixtureChannelValue3>,
        >,
    ) -> Result<(), FixtureError> {
        for (path, state) in self.fixture_states.iter_mut() {
            let fixture = patch.fixture(path)?;

            for (attribute, cf) in fixture.channel_functions() {
                let mut new_output_value = state.sources().get_attribute_value(
                    &fixture.path,
                    state,
                    attribute,
                    updatable_handler,
                    preset_handler,
                    timing_handler,
                )?;

                if highlight.is_some_and(|highlight| {
                    // all child fixtures should also be highlighted
                    highlight.has_fixture_with_level(path, FixturePathMatchLevel::TopLevel)
                }) {
                    if let Some(highlight) = cf.highlight() {
                        new_output_value = FixtureChannelValue3::discrete(highlight);
                    }
                }

                let output_value = state.cached_output_mut().get_mut(attribute).unwrap();

                if output_value.value == new_output_value {
                    continue;
                }

                output_value.update(new_output_value.clone());
                updated_output_values
                    .entry(*path)
                    .or_default()
                    .insert(*attribute, new_output_value.clone());
            }
        }

        Ok(())
    }

    pub fn submit_output_values(
        &mut self,
        value_queue_tx: &mpsc::Sender<ChannelValueQueueEntry>,
        patch: &Patch,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> Result<(), FixtureError> {
        for (path, state) in self.fixture_states.iter_mut() {
            let mut updated_values = HashMap::new();
            let fixture = patch.fixture(path)?;

            for (attribute, output_value) in state.cached_output_mut() {
                if !output_value.should_output(preset_handler) {
                    continue;
                }

                let discrete_value = output_value.value().clone().to_discrete(
                    fixture,
                    &attribute,
                    preset_handler,
                    timing_handler,
                );

                updated_values.insert(attribute.clone(), (discrete_value, None));
                output_value.reset();
            }

            if !updated_values.is_empty() {
                value_queue_tx
                    .send(ChannelValueQueueEntry {
                        fixture_path: *path,
                        values: updated_values,
                    })
                    .expect("Output channel has hung up");
            }
        }

        Ok(())
    }
}
