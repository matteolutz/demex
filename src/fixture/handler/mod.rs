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
    universe_output_state: HashMap<u16, [u8; 512]>,
}

impl FixtureHandler {
    pub fn new() -> Self {
        Self {
            fixtures: Vec::new(),
            state: HashMap::new(),
            outputs: Vec::new(),
            dirty_fixtures: BTreeSet::new(),
            universe_output_state: HashMap::new(),
        }
    }

    pub fn add_output(&mut self, output: Box<dyn DMXOutput>) {
        self.outputs.push(output);
    }

    pub fn add_fixture(&mut self, fixture: Fixture) -> Result<(), FixtureHandlerError> {
        if self.state.contains_key(&fixture.id) {
            return Err(FixtureHandlerError::FixtureAlreadyExists);
        }

        // TODO: check for overlapping DMX addresses

        self.state.insert(fixture.id, FixtureState::default());
        self.fixtures.push(fixture);
        Ok(())
    }

    fn assert_fixture_exists(&self, fixture_id: u32) -> Result<(), FixtureHandlerError> {
        if self.state.contains_key(&fixture_id) {
            Ok(())
        } else {
            Err(FixtureHandlerError::FixtureNotFound)
        }
    }

    pub fn set_fixture_state(
        &mut self,
        fixture_id: u32,
        new_state: FixtureState,
    ) -> Result<(), FixtureHandlerError> {
        self.assert_fixture_exists(fixture_id)?;

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

    pub fn go_home(&mut self, fixture_id: u32) -> Result<(), FixtureHandlerError> {
        self.assert_fixture_exists(fixture_id)?;

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

    pub fn update(&mut self) -> Result<(), FixtureHandlerError> {
        if !self.dirty_fixtures.is_empty() {
            let mut dirty_universes: BTreeSet<u16> = BTreeSet::new();

            for f in self.fixtures.iter() {
                let fixture_id = f.id();
                if self.dirty_fixtures.contains(&fixture_id) {
                    let fixture_state = self.state.get(&fixture_id).unwrap();
                    let fixture_data_packet = f.generate_data_packet(fixture_state.clone());

                    let universe = f.universe();
                    dirty_universes.insert(universe);

                    // insert universe if it doesn't exist
                    self.universe_output_state
                        .entry(universe)
                        .or_insert_with(|| [0; 512]);

                    let start_address = f.start_address() as usize;

                    // copy fixture data packet into universe data packet
                    let universe_data_packet =
                        self.universe_output_state.get_mut(&universe).unwrap();

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
        }

        Ok(())
    }
}
