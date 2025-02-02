use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::channel::value::FixtureChannelValue;

#[derive(Debug, Serialize, Deserialize, Clone, EguiProbe, Default)]
pub struct DemexFaderChannelOverride {
    from_value: Option<FixtureChannelValue>,
    to_value: FixtureChannelValue,
}

impl DemexFaderChannelOverride {
    pub fn new(from_value: Option<FixtureChannelValue>, to_value: FixtureChannelValue) -> Self {
        Self {
            from_value,
            to_value,
        }
    }

    pub fn from_value(&self) -> &Option<FixtureChannelValue> {
        &self.from_value
    }

    pub fn to_value(&self) -> &FixtureChannelValue {
        &self.to_value
    }
}
