use std::collections::HashMap;

use crate::fixture::channel::{FixtureChannel, FixtureId};

#[derive(Debug, Clone)]
pub struct Cue {
    data: HashMap<FixtureId, Vec<FixtureChannel>>,
}

impl Cue {
    pub fn new(data: HashMap<FixtureId, Vec<FixtureChannel>>) -> Self {
        Self { data }
    }

    pub fn data(&self) -> &HashMap<FixtureId, Vec<FixtureChannel>> {
        &self.data
    }

    pub fn data_for_fixture(&self, fixture_id: FixtureId) -> Option<&Vec<FixtureChannel>> {
        self.data.get(&fixture_id)
    }
}
