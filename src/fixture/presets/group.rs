use serde::{Deserialize, Serialize};

use crate::{ fixture::selection::FixtureSelection};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FixtureGroup {
    id: u32,
    name: String,
    fixture_selection: FixtureSelection,
}

impl FixtureGroup {
    pub fn new(id: u32, name: Option<String>, fixture_selection: FixtureSelection) -> Self {
        FixtureGroup {
            id,
            name: name.unwrap_or(format!("Group {}", id)),
            fixture_selection,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    pub fn fixture_selection(&self) -> &FixtureSelection {
        &self.fixture_selection
    }
}
