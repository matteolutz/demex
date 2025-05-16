use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    fixture::{
        channel3::channel_value::FixtureChannelValue3, value_source::FixtureChannelValueSource,
    },
    headless::sync::DemexSync,
};

use super::GdtfFixture;

#[derive(Debug, Serialize, Deserialize)]
pub struct GdtfFixtureSync {
    programmer_values: HashMap<String, FixtureChannelValue3>,
    sources: Vec<FixtureChannelValueSource>,
}

impl DemexSync for GdtfFixture {
    type Sync = GdtfFixtureSync;

    fn apply(&mut self, sync: Self::Sync) {
        self.programmer_values = sync.programmer_values;
        self.sources = sync.sources;
    }

    fn get_sync(&self) -> Self::Sync {
        GdtfFixtureSync {
            programmer_values: self.programmer_values.clone(),
            sources: self.sources.clone(),
        }
    }
}
