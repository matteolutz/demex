use std::collections::{BTreeSet, HashMap};

use crate::dmx::{DemexDmxOutput, DemexDmxOutputTrait};

use self::error::FixtureHandlerError;

use super::{
    gdtf::GdtfFixture, presets::PresetHandler, selection::FixtureSelection, timing::TimingHandler,
    updatables::UpdatableHandler,
};

pub mod error;

pub type FixtureTypeList = [gdtf::fixture_type::FixtureType];

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
    fixtures: Vec<GdtfFixture>,
    outputs: Vec<DemexDmxOutput>,
    universe_output_data: HashMap<u16, [u8; 512]>,
    grand_master: u8,
}

impl FixtureHandler {
    pub fn default_grandmaster_value() -> u8 {
        255
    }

    pub fn new(
        fixtures: Vec<GdtfFixture>,
        outputs: Vec<DemexDmxOutput>,
    ) -> Result<Self, FixtureHandlerError> {
        // check if the fixtures overlap

        let mut fixture_addresses: HashMap<u16, BTreeSet<u16>> = HashMap::new();

        for f in &fixtures {
            let start_address = f.start_address();
            let end_address = start_address + f.address_footprint() - 1;
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
            grand_master: Self::default_grandmaster_value(),
        })
    }

    pub fn fixture_immut(&self, fixture_id: u32) -> Option<&GdtfFixture> {
        self.fixtures.iter().find(|f| f.id() == fixture_id)
    }

    pub fn fixture(&mut self, fixture_id: u32) -> Option<&mut GdtfFixture> {
        self.fixtures.iter_mut().find(|f| f.id() == fixture_id)
    }

    pub fn selected_fixtures_mut(
        &mut self,
        fixture_selection: &FixtureSelection,
    ) -> Vec<&mut GdtfFixture> {
        self.fixtures
            .iter_mut()
            .filter(|fixture| fixture_selection.has_fixture(fixture.id()))
            .collect::<Vec<_>>()
    }

    pub fn fixtures(&self) -> &Vec<GdtfFixture> {
        &self.fixtures
    }

    pub fn fixtures_mut(&mut self) -> &mut Vec<GdtfFixture> {
        &mut self.fixtures
    }

    pub fn has_fixture(&self, id: u32) -> bool {
        self.fixtures.iter().any(|f| f.id() == id)
    }

    pub fn outputs(&self) -> &Vec<DemexDmxOutput> {
        &self.outputs
    }

    pub fn home_all(&mut self, clear_sources: bool) -> Result<(), FixtureHandlerError> {
        for f in self.fixtures.iter_mut() {
            f.home(clear_sources)
                .map_err(FixtureHandlerError::FixtureError)?;
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
        fixture_types: &FixtureTypeList,
        preset_handler: &PresetHandler,
        updatable_handler: &UpdatableHandler,
        timing_handler: &TimingHandler,
        _delta_time: f64,
        force: bool,
    ) -> Result<usize, FixtureHandlerError> {
        let mut dirty_universes: BTreeSet<u16> = BTreeSet::new();

        for f in &mut self.fixtures {
            let fixture_universe_offset = f.start_address() - 1;

            let data_packet = f
                .generate_data_packet(
                    fixture_types,
                    preset_handler,
                    updatable_handler,
                    timing_handler,
                    self.grand_master as f32 / 255.0,
                )
                .map_err(FixtureHandlerError::FixtureError)?;

            if !force
                && compare_universe_output_data(
                    self.universe_output_data.get(&f.universe()),
                    &data_packet,
                    fixture_universe_offset,
                )
            {
                continue;
            }

            let universe_data = self
                .universe_output_data
                .entry(f.universe())
                .or_insert_with(|| [0; 512]);

            write_universe_data(universe_data, &data_packet, fixture_universe_offset);

            dirty_universes.insert(f.universe());
        }

        for output in &mut self.outputs {
            for (universe, data) in &self.universe_output_data {
                if !dirty_universes.contains(universe) {
                    continue;
                }

                if let Err(err) = output.send(*universe, data) {
                    log::warn!(
                        "Failed to send data via {:?} for universe {}. Did the corresponding output thread panic?\n{}",
                        output, universe, err
                    );
                }
            }
        }

        Ok(dirty_universes.len())
    }
}
