use serde::{Deserialize, Serialize};

use super::{Fixture, SerializableFixturePatch};

#[derive(Serialize, Deserialize)]
pub struct Patch(Vec<SerializableFixturePatch>);

impl Patch {
    pub fn new(fixtures: Vec<SerializableFixturePatch>) -> Self {
        Patch(fixtures)
    }
}

impl From<Patch> for Vec<Fixture> {
    fn from(value: Patch) -> Self {
        value.0.into_iter().map(|f| f.into()).collect()
    }
}
