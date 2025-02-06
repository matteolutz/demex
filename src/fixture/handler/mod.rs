use std::collections::{BTreeSet, HashMap};

use crate::dmx::DMXOutput;

use self::error::FixtureHandlerError;

use super::{presets::PresetHandler, updatables::UpdatableHandler, Fixture};

pub mod error;

fn compare_universe_output_data(
    previous_data_option: Option<&[u8; 512]>,
    fixture_data: &[u8],
    fixture_universe_offset: u16,
) -> bool {
    if previous_data_option.is_none() {
        return false;
    }

    let previous_data = previous_data_option.unwrap();

    for (i, d) in fixture_data.iter().enumerate() {
        if previous_data[i + fixture_universe_offset as usize] != *d {
            return false;
        }
    }

    true
}

fn write_universe_data(
    universe_data: &mut [u8; 512],
    fixture_data: &[u8],
    fixture_universe_offset: u16,
) {
    for (i, d) in fixture_data.iter().enumerate() {
        universe_data[i + fixture_universe_offset as usize] = *d;
    }
}

#[derive(Debug)]
pub struct FixtureHandler {
    fixtures: Vec<Fixture>,
    outputs: Vec<Box<dyn DMXOutput + Send + Sync>>,
    universe_output_data: HashMap<u16, [u8; 512]>,
    grand_master: u8,
}

impl FixtureHandler {
    pub fn new(
        outputs: Vec<Box<dyn DMXOutput + Sync + Send>>,
        fixtures: Vec<Fixture>,
    ) -> Result<Self, FixtureHandlerError> {
        // check if the fixtures overlap

        let mut fixture_addresses: HashMap<u16, BTreeSet<u16>> = HashMap::new();

        for f in &fixtures {
            let start_address = f.start_address();
            let end_address = start_address + f.address_bandwidth() - 1;
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

        Ok(Self {
            universe_output_data: HashMap::with_capacity(fixtures.len()),
            fixtures,
            outputs,
            grand_master: 255,
        })
    }

    pub fn fixture_immut(&self, fixture_id: u32) -> Option<&Fixture> {
        self.fixtures.iter().find(|f| f.id() == fixture_id)
    }

    pub fn fixture(&mut self, fixture_id: u32) -> Option<&mut Fixture> {
        self.fixtures.iter_mut().find(|f| f.id() == fixture_id)
    }

    pub fn fixtures(&self) -> &Vec<Fixture> {
        &self.fixtures
    }

    pub fn home_all(&mut self) -> Result<(), FixtureHandlerError> {
        for f in self.fixtures.iter_mut() {
            f.home().map_err(FixtureHandlerError::FixtureError)?;
        }

        Ok(())
    }

    pub fn grand_master(&self) -> u8 {
        self.grand_master
    }

    pub fn grand_master_mut(&mut self) -> &mut u8 {
        &mut self.grand_master
    }

    pub fn update(
        &mut self,
        preset_handler: &PresetHandler,
        updatable_handler: &UpdatableHandler,
        _delta_time: f64,
    ) -> Result<(), FixtureHandlerError> {
        let mut dirty_universes: BTreeSet<u16> = BTreeSet::new();

        for f in &self.fixtures {
            let fixture_universe_offset = f.start_address() - 1;

            let data_packet = f
                .generate_data_packet(preset_handler, updatable_handler)
                .map_err(FixtureHandlerError::FixtureChannelError)?;

            let fixture_data = data_packet
                .iter()
                .map(|p| (*p as f32 * (self.grand_master as f32 / 255.0)) as u8)
                .collect::<Vec<u8>>();

            if compare_universe_output_data(
                self.universe_output_data.get(&f.universe()),
                &fixture_data,
                fixture_universe_offset,
            ) {
                continue;
            }

            let universe_data = self
                .universe_output_data
                .entry(f.universe())
                .or_insert_with(|| [0; 512]);

            write_universe_data(universe_data, &fixture_data, fixture_universe_offset);

            dirty_universes.insert(f.universe());
        }

        for output in self.outputs.iter_mut() {
            for (universe, data) in &self.universe_output_data {
                if !dirty_universes.contains(universe) {
                    continue;
                }

                output
                    .send(*universe, data)
                    .map_err(FixtureHandlerError::FixtureHandlerUpdateError)?;
            }
        }

        Ok(())
    }
}
