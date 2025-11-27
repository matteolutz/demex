use std::collections::HashMap;

use crate::{
    channel3::{
        channel_value::FixtureChannelValue3, channel_value_state::FixtureChannelOutputValue,
    },
    fixture::{GdtfFixturePatch, error::FixtureError},
    patch::Patch,
    value_source::FixtureChannelValueSource,
};

#[derive(Debug)]
pub struct FixtureState {
    programmer_values: HashMap<String, FixtureChannelValue3>,

    sources: Vec<FixtureChannelValueSource>,
    cached_output: HashMap<String, FixtureChannelOutputValue>,
}

impl FixtureState {
    pub fn sources(&self) -> &[FixtureChannelValueSource] {
        &self.sources
    }

    pub fn cached_output(&self) -> &HashMap<String, FixtureChannelOutputValue> {
        &self.cached_output
    }

    pub fn cached_output_mut(&mut self) -> &mut HashMap<String, FixtureChannelOutputValue> {
        &mut self.cached_output
    }

    pub fn get_programmer_value(
        &self,
        channel: &str,
    ) -> Result<&FixtureChannelValue3, FixtureError> {
        self.programmer_values
            .get(channel)
            .ok_or_else(|| FixtureError::GdtfChannelNotFound(channel.to_owned()))
    }
}

impl FixtureState {
    pub fn new(fixture: &GdtfFixturePatch, patch: &Patch) -> Self {
        let (_, dmx_mode) = patch
            .fixture_type_and_dmx_mode(fixture)
            .expect("Fixture type DMX mode not found");

        let programmer_values: HashMap<String, FixtureChannelValue3> = dmx_mode
            .dmx_channels
            .iter()
            .map(|channel| {
                (
                    channel.name().as_ref().to_owned(),
                    FixtureChannelValue3::home(),
                )
            })
            .collect();

        let cached_output = programmer_values
            .clone()
            .into_iter()
            .map(|(key, value)| (key, FixtureChannelOutputValue::new_changed(value)))
            .collect();

        Self {
            programmer_values,
            cached_output,
            sources: vec![FixtureChannelValueSource::Programmer],
        }
    }
}
