use std::collections::HashMap;

use demex_headless::sync::DemexSync;
use serde::{Deserialize, Serialize};

use crate::{
    channel3::{
        channel_value::{FixtureChannelValue3, FixtureChannelValue3Update},
        channel_value_discrete::FixtureChannelDiscreteValue,
        channel_value_state::FixtureChannelOutputValue,
        utils::dmx_value_to_f32,
    },
    fixture::{GdtfFixturePatch, error::FixtureError},
    patch::Patch,
    value_source::FixtureChannelValueSource,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct GdtfFixtureStateSync {
    output_values: HashMap<String, FixtureChannelValue3>,
}

impl DemexSync for FixtureState {
    type Sync = GdtfFixtureStateSync;

    fn apply(&mut self, sync: Self::Sync) {
        self.cached_output = sync
            .output_values
            .into_iter()
            .map(|(channel, value)| (channel, FixtureChannelOutputValue::new_changed(value)))
            .collect();
    }

    fn get_sync(&self) -> Self::Sync {
        GdtfFixtureStateSync {
            output_values: self
                .cached_output
                .iter()
                .map(|(channel, value)| (channel.clone(), value.value.clone()))
                .collect(),
        }
    }
}

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
    pub fn home(&mut self, clear_sources: bool) -> Result<(), FixtureError> {
        if clear_sources {
            // remove every source except the programmer
            self.sources.clear();
            self.sources.push(FixtureChannelValueSource::Programmer);
        }

        for value in self.programmer_values.values_mut() {
            *value = FixtureChannelValue3::home();
        }

        Ok(())
    }

    pub fn push_value_source(&mut self, value_source: FixtureChannelValueSource) {
        self.sources.retain(|source| !source.eq(&value_source));
        self.sources.push(value_source);
    }

    pub fn remove_value_source(&mut self, value_source: FixtureChannelValueSource) {
        self.sources.retain(|source| source != &value_source);
    }

    pub fn get_programmer_value(
        &self,
        channel: &str,
    ) -> Result<&FixtureChannelValue3, FixtureError> {
        self.programmer_values
            .get(channel)
            .ok_or_else(|| FixtureError::GdtfChannelNotFound(channel.to_owned()))
    }

    pub fn set_programmer_value(
        &mut self,
        patch: &Patch,
        fixture: &GdtfFixturePatch,
        channel: &str,
        value: FixtureChannelValue3,
    ) -> Result<(), FixtureError> {
        let (fixture_type, _) = patch.fixture_type_and_dmx_mode(fixture)?;

        let (_, logical_channel) = fixture.get_channel(patch, channel)?;
        let logical_channel_attribute = logical_channel
            .attribute(fixture_type)
            .ok_or_else(|| FixtureError::GdtfChannelHasNoAttribute(channel.to_owned()))?;

        let programmer_value = self
            .programmer_values
            .get_mut(channel)
            .ok_or_else(|| FixtureError::GdtfChannelNotFound(channel.to_owned()))?;
        *programmer_value = value.clone();

        if let Some(activation_group) =
            logical_channel_attribute.activation_group(&fixture_type.attribute_definitions)
        {
            for (dmx_channel, _) in fixture
                .channels(patch)?
                .filter(|(_, other_logical_channel)| {
                    *other_logical_channel != logical_channel
                        && other_logical_channel
                            .attribute(fixture_type)
                            .and_then(|attribute| {
                                attribute.activation_group(&fixture_type.attribute_definitions)
                            })
                            .is_some_and(|channel_activation_group| {
                                channel_activation_group == activation_group
                            })
                })
            {
                let channel_value = self
                    .programmer_values
                    .get_mut(dmx_channel.name().as_ref())
                    .unwrap();

                if channel_value.is_home() && !value.is_home() {
                    *channel_value =
                        FixtureChannelValue3::Discrete(FixtureChannelDiscreteValue::Discrete {
                            channel_function_idx: 0,
                            value: dmx_value_to_f32(
                                dmx_channel.logical_channels[0].channel_functions[0].default,
                            ),
                        });
                } else if !channel_value.is_home() && value.is_home() {
                    // set home value for function 0
                    *channel_value = FixtureChannelValue3::home();
                }
            }
        }

        Ok(())
    }

    pub fn update_programmer_attribute_matches_value(
        &mut self,
        patch: &Patch,
        fixture: &GdtfFixturePatch,
        filter: impl Fn(&str) -> bool,
        slider_val: FixtureChannelValue3Update,
    ) -> Result<(), FixtureError> {
        for (channel, _, _) in fixture.channels_for_attribute_matches(patch, filter)? {
            self.update_programmer_value(
                patch,
                fixture,
                channel.name().as_ref(),
                slider_val.clone(),
            )?;
        }

        Ok(())
    }

    pub fn update_programmer_value(
        &mut self,
        patch: &Patch,
        fixture: &GdtfFixturePatch,
        channel: &str,
        slider_val: FixtureChannelValue3Update,
    ) -> Result<(), FixtureError> {
        let programmer_value = self.get_programmer_value(channel)?;

        let channel_function_idx = match programmer_value {
            FixtureChannelValue3::Discrete(discrete) => match discrete {
                FixtureChannelDiscreteValue::Discrete {
                    channel_function_idx,
                    value: _,
                } => *channel_function_idx,
                FixtureChannelDiscreteValue::DiscreteSet {
                    channel_function_idx,
                    ..
                } => *channel_function_idx,
                _ => fixture.get_channel_initial_function_idx(patch, channel)?,
            },
            _ => fixture.get_channel_initial_function_idx(patch, channel)?,
        };

        self.set_programmer_value(
            patch,
            fixture,
            channel,
            slider_val.get_value(channel_function_idx),
        )
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
