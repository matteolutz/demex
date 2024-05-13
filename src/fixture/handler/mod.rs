use std::collections::{BTreeSet, HashMap};

use crate::dmx::DMXOutput;

use self::error::FixtureHandlerError;

use super::{state::FixtureState, Fixture};

pub mod error;

#[derive(Debug)]
pub struct FixtureHandler {
    fixtures: Vec<Fixture>,
    state: HashMap<u32, FixtureState>,
    outputs: Vec<Box<dyn DMXOutput>>,
    dirty_fixtures: BTreeSet<u32>,
    should_update: bool,
    universe_output_state: HashMap<u16, [u8; 512]>,
    grand_master: u8,
}

impl FixtureHandler {
    pub fn new(outputs: Vec<Box<dyn DMXOutput>>, fixtures: Vec<Fixture>) -> Self {
        let mut selff = Self {
            universe_output_state: HashMap::with_capacity(fixtures.len()),
            fixtures,
            state: HashMap::new(),
            outputs,
            dirty_fixtures: BTreeSet::new(),
            should_update: false,
            grand_master: 255,
        };

        // TODO: sanitize fixture input

        for f in &selff.fixtures {
            selff.state.insert(f.id, FixtureState::default());
        }

        selff
    }

    fn assert_fixture_exists(&self, fixture_id: u32) -> Result<(), FixtureHandlerError> {
        if self.state.contains_key(&fixture_id) {
            Ok(())
        } else {
            Err(FixtureHandlerError::FixtureNotFound(fixture_id))
        }
    }

    // only used within FixtureHandler
    fn set_fixture_state_unchecked(
        &mut self,
        fixture_id: u32,
        new_state: FixtureState,
    ) -> Result<(), FixtureHandlerError> {
        // don't update if the state is the same
        if let Some(fixture_state) = self.state.get(&fixture_id) {
            if fixture_state == &new_state {
                return Ok(());
            }
        }

        self.state.insert(fixture_id, new_state);
        self.dirty_fixtures.insert(fixture_id);
        Ok(())
    }

    pub fn set_fixture_state(
        &mut self,
        fixture_id: u32,
        new_state: FixtureState,
    ) -> Result<(), FixtureHandlerError> {
        self.assert_fixture_exists(fixture_id)?;

        self.set_fixture_state_unchecked(fixture_id, new_state)
    }

    pub fn go_home(&mut self, fixture_id: u32) -> Result<(), FixtureHandlerError> {
        self.set_fixture_state(fixture_id, FixtureState::default())
    }

    pub fn go_home_all(&mut self) -> Result<(), FixtureHandlerError> {
        let fixture_ids: Vec<u32> = self.state.keys().copied().collect();

        for fixture_id in fixture_ids {
            self.go_home(fixture_id)?;
        }

        Ok(())
    }

    pub fn fixture_state(&self, fixture_id: u32) -> Result<&FixtureState, FixtureHandlerError> {
        self.assert_fixture_exists(fixture_id)?;

        Ok(self.state.get(&fixture_id).unwrap())
    }

    pub fn fixtures(&self) -> &Vec<Fixture> {
        &self.fixtures
    }

    pub fn grand_master(&self) -> u8 {
        self.grand_master
    }

    pub fn set_grand_master(&mut self, grand_master: u8) {
        self.grand_master = grand_master;
        self.should_update = true;
    }

    pub fn update(&mut self) -> Result<(), FixtureHandlerError> {
        // update if there are dirty fixtures or if the update flag is set
        if !self.dirty_fixtures.is_empty() || self.should_update {
            let mut dirty_universes: BTreeSet<u16> = BTreeSet::new();

            for f in self.fixtures.iter() {
                let fixture_id = f.id();

                if self.should_update || self.dirty_fixtures.contains(&fixture_id) {
                    let fixture_state = self.state.get(&fixture_id).unwrap();

                    // generate the fixture data packet, translating the fixture state into a packet of bytes
                    let fixture_data_packet = f.generate_data_packet(
                        fixture_state.intensity_mult(self.grand_master() as f32 / 255.0),
                    );

                    let universe = f.universe();

                    // insert universe if it doesn't exist
                    self.universe_output_state
                        .entry(universe)
                        .or_insert_with(|| [0; 512]);

                    let start_address = f.start_address() as usize;

                    // copy fixture data packet into universe data packet
                    let universe_data_packet =
                        self.universe_output_state.get_mut(&universe).unwrap();

                    // if the manual update flag is set (e.g. through a GM value change), check if the fixture is really dirty
                    if self.should_update {
                        let mut is_dirty = false;

                        for i in 0..fixture_data_packet.len() {
                            if fixture_data_packet[i] != universe_data_packet[start_address - 1 + i]
                            {
                                is_dirty = true;
                                break;
                            }
                        }

                        if !is_dirty {
                            continue;
                        }
                    }

                    dirty_universes.insert(universe);

                    for (i, byte) in fixture_data_packet.iter().enumerate() {
                        universe_data_packet[start_address - 1 + i] = *byte;
                    }

                    self.dirty_fixtures.remove(&fixture_id);
                }
            }

            for output in self.outputs.iter_mut() {
                for (universe, data) in self.universe_output_state.iter() {
                    if !dirty_universes.contains(universe) {
                        continue;
                    }

                    output
                        .send(*universe, data)
                        .map_err(FixtureHandlerError::FixtureHandlerUpdateError)?;
                }
            }

            self.dirty_fixtures.clear();
            self.should_update = false;
        }

        Ok(())
    }
}
