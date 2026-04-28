use std::{collections::HashMap, ops::Range};

use serde::{Deserialize, Serialize};

use demex_dmx::{DemexDmxOutputConfig, address::DmxAddress};
use uuid::Uuid;

use crate::{
    channel3::attribute::FixtureChannel3Attribute,
    engine::component::Component,
    fixture::{
        Fixture, FixtureId, FixturePath, GdtfFixturePatch, builder::FixtureBuilder,
        error::FixtureError,
    },
    layout::FixtureLayoutPool,
};

pub type FixtureTypeList = [gdtf::fixture_type::FixtureType];

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SerializablePatch {
    fixtures: Vec<GdtfFixturePatch>,
    layout_pool: FixtureLayoutPool,
    outputs: Vec<DemexDmxOutputConfig>,
}

impl SerializablePatch {
    pub fn into_patch(self, fixture_types: Vec<gdtf::fixture_type::FixtureType>) -> Patch {
        let mut dmx_map: HashMap<DmxAddress, (FixtureId, FixtureChannel3Attribute)> =
            HashMap::new();

        let fixtures = self
            .fixtures
            .clone()
            .into_iter()
            .flat_map(|fixture| {
                let builder = FixtureBuilder::from_patch(fixture, &fixture_types)
                    .unwrap()
                    .should_collapse(true);

                let (fixtures, mut root_fixture_dmx_map) = builder.build_fixture_tree().unwrap();

                if let Some(root_fixture) = fixtures.first() {
                    for (address, attribute) in root_fixture_dmx_map.drain() {
                        let prev = dmx_map.insert(address, (root_fixture.path.root(), attribute));

                        if let Some(prev) = prev {
                            // TODO: don't panic here

                            panic!(
                                "DMX Address range overlap. Conflict on address {}: {:?} vs {:?}",
                                address,
                                prev,
                                (root_fixture.path.root(), attribute)
                            )
                        }
                    }
                }

                fixtures
            })
            .map(|f| (f.path(), f))
            .collect::<HashMap<_, _>>();

        Patch {
            fixtures,
            dmx_map,
            fixture_types,
            patch: self,
        }
    }
}

impl Component for Patch {}

#[derive(Debug, Clone, Default)]
pub struct Patch {
    fixtures: HashMap<FixturePath, Fixture>,
    fixture_types: Vec<gdtf::fixture_type::FixtureType>,
    pub(crate) dmx_map: HashMap<DmxAddress, (FixtureId, FixtureChannel3Attribute)>,
    pub(crate) patch: SerializablePatch,
}

impl Patch {
    pub fn fixtures(&self) -> impl Iterator<Item = &Fixture> {
        self.fixtures.values()
    }

    pub fn fixture(&self, path: &FixturePath) -> Result<&Fixture, FixtureError> {
        self.fixtures.get(path).ok_or(FixtureError::NotFound(*path))
    }

    pub fn fixture_types(&self) -> &FixtureTypeList {
        &self.fixture_types
    }

    pub fn fixture_types_mut(&mut self) -> &mut Vec<gdtf::fixture_type::FixtureType> {
        &mut self.fixture_types
    }

    pub fn fixture_type(&self, id: Uuid) -> Option<&gdtf::fixture_type::FixtureType> {
        self.fixture_types
            .iter()
            .find(|ft| ft.fixture_type_id == id)
    }

    pub fn layout_pool(&self) -> &FixtureLayoutPool {
        &self.patch.layout_pool
    }

    pub fn output_configs(&self) -> &[DemexDmxOutputConfig] {
        &self.patch.outputs
    }

    pub fn output_configs_mut(&mut self) -> &mut Vec<DemexDmxOutputConfig> {
        &mut self.patch.outputs
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
