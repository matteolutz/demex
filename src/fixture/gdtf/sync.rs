use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    fixture::channel3::{
        channel_value::FixtureChannelValue3, channel_value_state::FixtureChannelValue3State,
    },
    headless::sync::DemexSync,
};

use super::GdtfFixture;

#[derive(Debug, Serialize, Deserialize)]
pub struct GdtfFixtureSync {
    output_values: HashMap<String, FixtureChannelValue3>,
}

impl DemexSync for GdtfFixture {
    type Sync = GdtfFixtureSync;

    fn apply(&mut self, sync: Self::Sync) {
        self.outputs_values = sync
            .output_values
            .into_iter()
            .map(|(channel, value)| (channel, (value, FixtureChannelValue3State::new_changed())))
            .collect();
    }

    fn get_sync(&self) -> Self::Sync {
        GdtfFixtureSync {
            output_values: self
                .outputs_values
                .iter()
                .map(|(channel, (value, _))| (channel.clone(), value.clone()))
                .collect(),
        }
    }
}
