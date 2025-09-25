use demex_headless::sync::DemexSync;
use serde::{Deserialize, Serialize};

use crate::{fixture::handler::sync::FixtureHandlerSync, show::context::ShowContext};

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
