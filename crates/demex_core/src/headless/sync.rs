use serde::{Deserialize, Serialize};

use crate::show::context::ShowContext;

#[derive(Debug, Serialize, Deserialize)]
pub struct DemexProtoSync {
    // fixture_handler: FixtureHandlerSync,
}

impl DemexProtoSync {
    pub fn get(_show_context: &ShowContext) -> Self {
        Self {
            // fixture_handler: show_context.fixture_handler.read().get_sync(),
        }
    }

    pub fn apply(self, _show_context: &ShowContext) {
        /*show_context
        .fixture_handler
        .write() .apply(self.fixture_handler);*/
    }
}
