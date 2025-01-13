use serde::{Deserialize, Serialize};

use super::{layout::FixtureLayout, Fixture, SerializableFixturePatch};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    fixtures: Vec<SerializableFixturePatch>,
    layout: FixtureLayout,
}

impl Patch {
    pub fn new(fixtures: Vec<SerializableFixturePatch>) -> Self {
        Self {
            fixtures,
            layout: FixtureLayout::new(Vec::new()),
        }
    }

    pub fn layout(&self) -> &FixtureLayout {
        &self.layout
    }
}

impl From<Patch> for Vec<Fixture> {
    fn from(value: Patch) -> Self {
        value.fixtures.into_iter().map(|f| f.into()).collect()
    }
}
