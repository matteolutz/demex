use std::collections::HashMap;

use crate::{
    channel3::{
        attribute::FixtureChannel3Attribute, channel_value::FixtureChannelValue3,
        channel_value_discrete::FixtureChannelDiscreteValue,
        channel_value_state::FixtureChannelOutputValue,
    },
    fixture::{Fixture, error::FixtureError},
    value_source::FixtureChannelValueSource,
};

/*
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
*/

#[derive(Debug, Clone)]
pub struct FixtureState {
    programmer_values: HashMap<FixtureChannel3Attribute, FixtureChannelValue3>,

    sources: Vec<FixtureChannelValueSource>,
    cached_output: HashMap<FixtureChannel3Attribute, FixtureChannelOutputValue>,
}

impl FixtureState {
    pub fn sources(&self) -> &[FixtureChannelValueSource] {
        &self.sources
    }

    pub fn cached_output_moved(
        self,
    ) -> HashMap<FixtureChannel3Attribute, FixtureChannelOutputValue> {
        self.cached_output
    }

    pub fn cached_output(&self) -> &HashMap<FixtureChannel3Attribute, FixtureChannelOutputValue> {
        &self.cached_output
    }

    pub fn cached_output_mut(
        &mut self,
    ) -> &mut HashMap<FixtureChannel3Attribute, FixtureChannelOutputValue> {
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
        attribute: &FixtureChannel3Attribute,
    ) -> Result<&FixtureChannelValue3, FixtureError> {
        self.programmer_values
            .get(attribute)
            .ok_or_else(|| FixtureError::GdtfAttributeNotFound(*attribute))
    }

    pub fn set_programmer_value(
        &mut self,
        fixture: &Fixture,
        attribute: &FixtureChannel3Attribute,
        value: FixtureChannelValue3,
    ) -> Result<(), FixtureError> {
        let channel_function = fixture
            .channel_function(attribute)
            .ok_or(FixtureError::GdtfAttributeNotFound(*attribute))?;

        let programmer_value = self
            .programmer_values
            .get_mut(attribute)
            .ok_or_else(|| FixtureError::GdtfAttributeNotFound(*attribute))?;
        *programmer_value = value.clone();

        if let Some(activation_group) = channel_function.activation_group.as_ref() {
            for (other_attribute, other_channel_function) in
                fixture
                    .channel_functions()
                    .filter(|(other_attribute, other_channel_function)| {
                        attribute != *other_attribute
                            && other_channel_function
                                .activation_group
                                .as_ref()
                                .is_some_and(|other_ag| other_ag == activation_group)
                    })
            {
                let channel_value = self.programmer_values.entry(*other_attribute).or_default();

                if channel_value.is_home() && !value.is_home() {
                    *channel_value =
                        FixtureChannelValue3::Discrete(FixtureChannelDiscreteValue::Discrete {
                            value: other_channel_function.unprojected_default(),
                        });
                } else if !channel_value.is_home() && value.is_home() {
                    // set home value for function 0
                    *channel_value = FixtureChannelValue3::home();
                }
            }
        }

        Ok(())
    }
}

impl FixtureState {
    pub fn new(fixture: &Fixture) -> Self {
        let programmer_values: HashMap<FixtureChannel3Attribute, FixtureChannelValue3> = fixture
            .channel_functions()
            .map(|(attribute, _)| (*attribute, FixtureChannelValue3::home()))
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
