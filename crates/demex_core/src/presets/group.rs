use serde::{Deserialize, Serialize};

use crate::{implement_set_property, pool::PoolItem, selection::FixtureSelection};

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

    pub fn fixture_selection_mut(&mut self) -> &mut FixtureSelection {
        &mut self.fixture_selection
    }
}

#[derive(strum_macros::EnumString, strum_macros::Display)]
pub enum FixtureGroupProperty {
    Name,
}

implement_set_property! {
    for FixtureGroup with FixtureGroupProperty,

    Name => name as String
}

impl From<&FixtureGroup> for PoolItem {
    fn from(value: &FixtureGroup) -> Self {
        PoolItem {
            id: value.id,
            name: value.name.clone().into(),
            colors: None,
            flags: 0,
        }
    }
}
