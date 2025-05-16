use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{fixture::gdtf::sync::GdtfFixtureSync, headless::sync::DemexSync};

use super::FixtureHandler;

#[derive(Debug, Serialize, Deserialize)]
pub struct FixtureHandlerSync {
    fixtures: HashMap<u32, GdtfFixtureSync>,
}

impl DemexSync for FixtureHandler {
    type Sync = FixtureHandlerSync;

    fn apply(&mut self, sync: Self::Sync) {
        for (fixture_id, sync) in sync.fixtures.into_iter() {
            let fixture = self.fixture(fixture_id);

            if let Some(fixture) = fixture {
                fixture.apply(sync);
            }
        }
    }

    fn get_sync(&self) -> Self::Sync {
        FixtureHandlerSync {
            fixtures: self
                .fixtures
                .iter()
                .map(|fixture| (fixture.id(), fixture.get_sync()))
                .collect(),
        }
    }
}
