use std::collections::HashMap;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::dmx::{DemexDmxOutput, DemexDmxOutputConfig};

use super::{
    channel2::{channel_type::FixtureChannelType, feature::feature_config::FixtureFeatureConfig},
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Patch {
    fixtures: Vec<SerializableFixturePatch>,
    fixture_types: HashMap<String, FixturePatchType>,
    layout: FixtureLayout,
    outputs: Vec<DemexDmxOutputConfig>,
}

impl Patch {
    pub fn fixtures(&self) -> &[SerializableFixturePatch] {
        &self.fixtures
    }

    pub fn fixture_types(&self) -> &HashMap<String, FixturePatchType> {
        &self.fixture_types
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
}

impl From<Patch> for Vec<Fixture> {
    fn from(value: Patch) -> Self {
        value
            .fixtures
            .into_iter()
            .map(|f| f.try_into_fixture(&value.fixture_types).unwrap())
            .collect()
    }
}

impl From<&Patch> for Vec<Fixture> {
    fn from(value: &Patch) -> Self {
        value
            .fixtures
            .iter()
            .cloned()
            .map(|f| f.try_into_fixture(&value.fixture_types).unwrap())
            .collect()
    }
}

impl From<&Patch> for Vec<DemexDmxOutput> {
    fn from(value: &Patch) -> Self {
        value.outputs.iter().cloned().map_into().collect()
    }
}

impl From<Patch> for (Vec<Fixture>, Vec<DemexDmxOutput>) {
    fn from(value: Patch) -> Self {
        (
            value
                .fixtures
                .into_iter()
                .map(|f| f.try_into_fixture(&value.fixture_types).unwrap())
                .collect(),
            value.outputs.into_iter().map_into().collect(),
        )
    }
}
