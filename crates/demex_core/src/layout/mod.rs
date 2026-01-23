use std::ops::Deref;

use serde::{Deserialize, Serialize};

mod decoration;
pub use decoration::*;

mod entry;
pub use entry::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureLayout {
    name: String,
    fixtures: Vec<FixtureLayoutEntry>,
    decorations: Vec<FixtureLayoutDecoration>,
}

impl FixtureLayout {
    pub fn new(name: String) -> Self {
        Self {
            name,
            fixtures: Vec::new(),
            decorations: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn decorations(&self) -> &[FixtureLayoutDecoration] {
        &self.decorations
    }

    pub fn fixtures(&self) -> &[FixtureLayoutEntry] {
        &self.fixtures
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FixtureLayoutPool {
    layouts: Vec<FixtureLayout>,
}

impl Deref for FixtureLayoutPool {
    type Target = [FixtureLayout];

    fn deref(&self) -> &Self::Target {
        &self.layouts
    }
}
