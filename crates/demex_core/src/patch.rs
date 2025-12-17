use std::{collections::HashMap, ops::Range};

use serde::{Deserialize, Serialize};

use demex_dmx::DemexDmxOutputConfig;
use uuid::Uuid;

use crate::{
    engine::component::Component,
    fixture::{
        Fixture, FixturePath, GdtfFixturePatch, builder::FixtureBuilder, error::FixtureError,
    },
};

use super::layout::FixtureLayout;

pub type FixtureTypeList = [gdtf::fixture_type::FixtureType];

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SerializablePatch {
    fixtures: Vec<GdtfFixturePatch>,
    layout: FixtureLayout,
    outputs: Vec<DemexDmxOutputConfig>,
}

impl SerializablePatch {
    pub fn into_patch(self, fixture_types: Vec<gdtf::fixture_type::FixtureType>) -> Patch {
        Patch {
            fixtures: self
                .fixtures
                .into_iter()
                .flat_map(|fixture| {
                    let builder = FixtureBuilder::from_patch(fixture, &fixture_types)
                        .unwrap()
                        .should_collapse(true);
                    builder.build_fixture_tree().unwrap()
                })
                .map(|f| (f.path(), f))
                .collect::<HashMap<_, _>>(),
            fixture_types,
            layout: self.layout,
            outputs: self.outputs,
        }
    }
}

impl Component for Patch {}

#[derive(Debug, Clone, Default)]
pub struct Patch {
    fixtures: HashMap<FixturePath, Fixture>,
    fixture_types: Vec<gdtf::fixture_type::FixtureType>,
    layout: FixtureLayout,
    outputs: Vec<DemexDmxOutputConfig>,
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

    /*
    pub fn fixture_type_and_dmx_mode<'a>(
        &'a self,
        fixture: &GdtfFixturePatch,
    ) -> Result<
        (
            &'a gdtf::fixture_type::FixtureType,
            &'a gdtf::dmx_mode::DmxMode,
        ),
        FixtureError,
    > {
        let fixture_type = self
            .fixture_types
            .iter()
            .find(|ft| ft.fixture_type_id == fixture.fixture_type_id)
            .ok_or_else(|| FixtureError::GdtfFixtureTypeNotFound(fixture.fixture_type_id))?;

        let dmx_mode = fixture_type
            .dmx_mode(&fixture.fixture_type_dmx_mode)
            .ok_or(FixtureError::GdtfFixtureDmxModeNotFound(
                fixture.fixture_type_dmx_mode.clone(),
            ))?;

        Ok((fixture_type, dmx_mode))
    }

    pub fn fixture_type_and_dmx_mode_by_id<'a>(
        &'a self,
        fixture_id: u32,
    ) -> Result<
        (
            &'a gdtf::fixture_type::FixtureType,
            &'a gdtf::dmx_mode::DmxMode,
            &'a GdtfFixturePatch,
        ),
        FixtureError,
    > {
        let fixture = self.fixture(fixture_id)?;
        self.fixture_type_and_dmx_mode(fixture)
            .map(|(t, m)| (t, m, fixture))
    }
    */

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
