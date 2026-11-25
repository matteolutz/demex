use crate::{channel3::channel_value::FixtureChannelValue3, presets::PresetHandler};

#[derive(Debug, Clone, Default)]
pub struct FixtureChannelOutputValue {
    pub value: FixtureChannelValue3,

    /// When set to true, this will cause the output value
    /// to be submitted to the output thread in the next iteration
    pub was_updated: bool,

    /// When set to true, this will cause the output value
    /// set to `was_updated` in the next iteration
    pub force_update: bool,
}

impl FixtureChannelOutputValue {
    pub fn should_output(&self, preset_handler: &PresetHandler) -> bool {
        self.value.should_output(self, preset_handler)
    }

    pub fn new_changed(value: FixtureChannelValue3) -> Self {
        Self {
            was_updated: true,
            force_update: true,
            value,
        }
    }

    pub fn reset(&mut self) {
        self.was_updated = false;
        self.force_update = false;
    }

    pub fn update(&mut self, value: FixtureChannelValue3) {
        self.was_updated = true;
        self.force_update = false;
        self.value = value;
    }

    pub fn value(&self) -> &FixtureChannelValue3 {
        &self.value
    }
}
