use std::{collections::HashMap, ops::Range};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    dmx::{DemexDmxOutput, DemexDmxOutputConfig},
    utils::range::ranges_overlap,
};

use super::{
    channel2::{channel_type::FixtureChannelType, feature::feature_config::FixtureFeatureConfig},
    gdtf::{GdtfFixture, GdtfFixturePatch},
    layout::FixtureLayout,
    Fixture, SerializableFixturePatch,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FixtureTypeAndMode {
    pub name: String,
    pub mode: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixturePatchTypeMode {
    pub name: String,
    pub channel_types: Vec<FixtureChannelType>,
    pub toggle_flags: Vec<HashMap<String, u8>>,
    pub feature_configs: Vec<FixtureFeatureConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixturePatchType {
    pub name: String,
    pub modes: HashMap<u32, FixturePatchTypeMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureFile {
    pub id: String,
    pub config: FixturePatchType,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Patch {
    fixtures: Vec<GdtfFixturePatch>,
    fixture_types: HashMap<String, FixturePatchType>,
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

    pub fn fixture_types(&self) -> &HashMap<String, FixturePatchType> {
        &self.fixture_types
    }

    pub fn fixture_types_mut(&mut self) -> &mut HashMap<String, FixturePatchType> {
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

    pub fn is_address_range_unpatched(&self, address_range: Range<u16>, universe: u16) -> bool {
        todo!();
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
    }
}

impl From<&Patch> for Vec<DemexDmxOutput> {
    fn from(value: &Patch) -> Self {
        value.outputs.iter().cloned().map_into().collect()
    }
}

impl Patch {
    pub fn into_fixures_and_outputs<'a>(
        self,
        fixture_types: &[gdtf::fixture_type::FixtureType],
    ) -> (Vec<GdtfFixture>, Vec<DemexDmxOutput>) {
        (
            self.fixtures
                .into_iter()
                .map(|f| f.into_fixture(fixture_types).unwrap())
                .collect(),
            self.outputs.into_iter().map_into().collect(),
        )
    }
}
