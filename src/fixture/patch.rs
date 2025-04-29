use std::ops::Range;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::dmx::{DemexDmxOutput, DemexDmxOutputConfig};

use super::{
    gdtf::{GdtfFixture, GdtfFixturePatch},
    handler::FixtureTypeList,
    layout::FixtureLayout,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SerializablePatch {
    fixtures: Vec<GdtfFixturePatch>,
    layout: FixtureLayout,
    outputs: Vec<DemexDmxOutputConfig>,
}

impl SerializablePatch {
    pub fn into_patch(self, fixture_types: Vec<gdtf::fixture_type::FixtureType>) -> Patch {
        Patch {
            fixtures: self.fixtures,
            fixture_types,
            layout: self.layout,
            outputs: self.outputs,
        }
    }

    pub fn from_patch(patch: &Patch) -> Self {
        SerializablePatch {
            fixtures: patch.fixtures.clone(),
            layout: patch.layout.clone(),
            outputs: patch.outputs.clone(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Patch {
    fixtures: Vec<GdtfFixturePatch>,
    fixture_types: Vec<gdtf::fixture_type::FixtureType>,
    layout: FixtureLayout,
    outputs: Vec<DemexDmxOutputConfig>,
}

impl Patch {
    pub fn fixtures(&self) -> &[GdtfFixturePatch] {
        &self.fixtures
    }

    pub fn fixtures_mut(&mut self) -> &mut Vec<GdtfFixturePatch> {
        &mut self.fixtures
    }

    pub fn fixture_types(&self) -> &FixtureTypeList {
        &self.fixture_types
    }

    pub fn fixture_types_mut(&mut self) -> &mut Vec<gdtf::fixture_type::FixtureType> {
        &mut self.fixture_types
    }

    pub fn layout(&self) -> &FixtureLayout {
        &self.layout
    }

    pub fn output_configs(&self) -> &[DemexDmxOutputConfig] {
        &self.outputs
    }

    pub fn output_configs_mut(&mut self) -> &mut Vec<DemexDmxOutputConfig> {
        &mut self.outputs
    }

    pub fn is_address_range_unpatched(&self, _address_range: Range<u16>, _universe: u16) -> bool {
        todo!();
        /*
        for fixture in self.fixtures.iter().filter(|f| f.universe == universe) {
            let fixture_type = self.fixture_types().get(&fixture.fixture_type).unwrap();
            let fixture_mode = fixture_type.modes.get(&fixture.fixture_mode).unwrap();

            let fixture_range = fixture.start_address
                ..(fixture.start_address + fixture_mode.channel_types.len() as u16);

            if ranges_overlap(fixture_range, address_range.clone()) {
                return false;
            }
        }
        true
        */
    }
}

impl From<&Patch> for Vec<DemexDmxOutput> {
    fn from(value: &Patch) -> Self {
        value.outputs.iter().cloned().map_into().collect()
    }
}

impl Patch {
    pub fn into_fixures_and_outputs(&self) -> (Vec<GdtfFixture>, Vec<DemexDmxOutput>) {
        (
            self.fixtures
                .clone()
                .into_iter()
                .map(|f| f.into_fixture(&self.fixture_types).unwrap())
                .collect(),
            self.outputs.clone().into_iter().map_into().collect(),
        )
    }
}
