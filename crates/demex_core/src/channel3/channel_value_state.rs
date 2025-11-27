use crate::{channel3::channel_value::FixtureChannelValue3, presets::PresetHandler};

#[derive(Debug, Clone, Default)]
pub struct FixtureChannelOutputValue {
    pub value: FixtureChannelValue3,
    pub was_updated: bool,
}

impl FixtureChannelOutputValue {
    pub fn should_output(&self, preset_handler: &PresetHandler) -> bool {
        self.value.should_output(self, preset_handler)
    }

    pub fn new_changed(value: FixtureChannelValue3) -> Self {
        Self {
            was_updated: true,
            value,
        }
    }

    pub fn reset(&mut self) {
        self.was_updated = false;
    }

    pub fn update(&mut self, value: FixtureChannelValue3) {
        self.was_updated = true;
        self.value = value;
    }

    pub fn value(&self) -> &FixtureChannelValue3 {
        &self.value
    }
}
