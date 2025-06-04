use serde::{Deserialize, Serialize};

use crate::{fixture::handler::sync::FixtureHandlerSync, show::context::ShowContext};

pub trait DemexSync {
    type Sync;

    fn apply(&mut self, sync: Self::Sync);
    fn get_sync(&self) -> Self::Sync;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DemexProtoSync {
    fixture_handler: FixtureHandlerSync,
}

impl DemexProtoSync {
    pub fn get(show_context: &ShowContext) -> Self {
        Self {
            fixture_handler: show_context.fixture_handler.read().get_sync(),
        }
    }

    pub fn apply(self, show_context: &ShowContext) {
        show_context
            .fixture_handler
            .write()
            .apply(self.fixture_handler);
    }
}
