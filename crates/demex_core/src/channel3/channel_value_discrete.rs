use std::collections::HashMap;

use gdtf::values::DmxValue;
use serde::{Deserialize, Serialize};

use crate::{
    channel3::utils::dmx_value_to_f32,
    fixture::{GdtfFixture, handler::FixtureTypeList},
};

use super::utils::{max_value, mix_dmx_value, multiply_dmx_value, multiply_dmx_value_f32};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum FixtureChannelDiscreteValue {
    #[default]
    Home,

    Discrete {
        channel_function_idx: usize,
        value: f32,
    },

    DiscreteSet {
        channel_function_idx: usize,
        channel_set: String,
    },

    Mix {
        a: Box<Self>,
        b: Box<Self>,
        mix: f32,
    },
}

impl FixtureChannelDiscreteValue {
    pub fn get_as_display(
        &self,
        fixture: &GdtfFixture,
        fixture_types: &FixtureTypeList,
        channel_name: &str,
    ) -> (usize, f32) {
        match self {
            Self::Home => {
                if let Ok((dmx_channel, _)) = fixture.get_channel(fixture_types, channel_name) {
                    let (logical_channel, function, default_dmx) = dmx_channel
                        .initial_function()
                        .map(|(logical_channel, function)| {
                            (logical_channel, function, function.default)
                        })
                        .unwrap();

                    let channel_function_idx = logical_channel
                        .channel_functions
                        .iter()
                        .position(|ft| ft == function)
                        .unwrap_or_default();

                    (channel_function_idx, dmx_value_to_f32(default_dmx))
                } else {
                    (0, 0.0)
                }
            }
            Self::Discrete {
                channel_function_idx,
                value,
            } => (*channel_function_idx, *value),
            Self::DiscreteSet {
                channel_function_idx,
                channel_set,
            } => {
                if let Ok((dmx_channel, _)) = fixture.get_channel(fixture_types, channel_name) {
                    let logical_channel = &dmx_channel.logical_channels[0];

                    let channel_function =
                        &logical_channel.channel_functions[*channel_function_idx];

                    let channel_function_from = dmx_value_to_f32(channel_function.dmx_from);
                    let channel_function_to = logical_channel
                        .channel_functions
                        .get(*channel_function_idx + 1)
                        .map(|channel_function| dmx_value_to_f32(channel_function.dmx_from))
                        .unwrap_or(1.0);

                    let channel_set_value = channel_function
                        .channel_set(channel_set)
                        .map(|channel_set| dmx_value_to_f32(channel_set.dmx_from))
                        .map(|channel_set_from_value| {
                            (channel_set_from_value - channel_function_from)
                                / (channel_function_to - channel_function_from)
                        })
                        .unwrap_or(0.0);

                    (*channel_function_idx, channel_set_value)
                } else {
                    (0, 0.0)
                }
            }
            Self::Mix { a, b, mix } => {
                let (a_idx, a_val) = a.get_as_display(fixture, fixture_types, channel_name);
                let (b_idx, b_val) = b.get_as_display(fixture, fixture_types, channel_name);

                if a_idx == b_idx {
                    (a_idx, (a_val * (1.0 - mix)) + (b_val * mix))
                } else if *mix < 0.5 {
                    (a_idx, a_val)
                } else {
                    (b_idx, b_val)
                }
            }
        }
    }
}

impl FixtureChannelDiscreteValue {
    fn find_multiply_relation(
        fixture: &GdtfFixture,
        fixture_types: &FixtureTypeList,
        dmx_mode: &gdtf::dmx_mode::DmxMode,
        dynamic_data: &mut HashMap<String, DmxValue>,
        values: &HashMap<String, FixtureChannelDiscreteValue>,
        channel_function: &gdtf::dmx_mode::ChannelFunction,
        grand_master: f32,
    ) -> Option<gdtf::values::DmxValue> {
        let relation = dmx_mode.relations.iter().find(|rel| {
            rel.follower(dmx_mode)
                .is_some_and(|(_, _, rel_function)| rel_function == channel_function)
        });
        relation.map(|rel| {
            let relation_master = rel.master(dmx_mode).unwrap();

            let relation_master_value = values.get(relation_master.name().as_ref()).unwrap();

            let value = relation_master_value
                ._to_dmx(
                    fixture,
                    fixture_types,
                    dmx_mode,
                    relation_master,
                    dynamic_data,
                    values,
                    grand_master,
                )
                .unwrap();
            dynamic_data.insert(relation_master.name().as_ref().to_string(), value);
            value
        })
    }

    /// Converts the channel value to a DMX value (0.0..=1.0)
    pub fn to_dmx(
        &self,
        fixture_types: &FixtureTypeList,
        fixture: &GdtfFixture,
        dmx_channel: &gdtf::dmx_mode::DmxChannel,
        dynamic_data: &mut HashMap<String, DmxValue>,
        grand_master: f32,
    ) -> Option<gdtf::values::DmxValue> {
        let (_, dmx_mode) = fixture.fixture_type_and_dmx_mode(fixture_types).ok()?;
        let values = fixture.programmer_values();

        self._to_dmx(
            fixture,
            fixture_types,
            dmx_mode,
            dmx_channel,
            dynamic_data,
            values,
            grand_master,
        )
    }

    fn _to_dmx(
        &self,
        fixture: &GdtfFixture,
        fixture_types: &FixtureTypeList,
        dmx_mode: &gdtf::dmx_mode::DmxMode,
        dmx_channel: &gdtf::dmx_mode::DmxChannel,
        dynamic_data: &mut HashMap<String, DmxValue>,
        values: &HashMap<String, FixtureChannelDiscreteValue>,
        grand_master: f32,
    ) -> Option<gdtf::values::DmxValue> {
        let logical_channel = &dmx_channel.logical_channels[0];

        if let Some(dynamic_value) = dynamic_data.get(dmx_channel.name().as_ref()) {
            return Some(*dynamic_value);
        }

        let value = match self {
            Self::Home => dmx_channel.initial_function().map(|(_, f)| {
                if let Some(relation_value) = Self::find_multiply_relation(
                    fixture,
                    fixture_types,
                    dmx_mode,
                    dynamic_data,
                    values,
                    f,
                    grand_master,
                ) {
                    multiply_dmx_value(f.default, relation_value)
                } else {
                    f.default
                }
            }),
            Self::DiscreteSet {
                channel_function_idx,
                channel_set,
            } => {
                let channel_function = &logical_channel.channel_functions[*channel_function_idx];

                let value = channel_function
                    .channel_set(channel_set)
                    .map(|channel_set| channel_set.dmx_from);

                if let Some(relation_value) = Self::find_multiply_relation(
                    fixture,
                    fixture_types,
                    dmx_mode,
                    dynamic_data,
                    values,
                    channel_function,
                    grand_master,
                ) {
                    value.map(|val| multiply_dmx_value(val, relation_value))
                } else {
                    value
                }
            }
            Self::Discrete {
                channel_function_idx,
                value,
            } => {
                let channel_function = &logical_channel.channel_functions[*channel_function_idx];

                let n_bytes = channel_function.dmx_from.bytes();
                let dmx_from = channel_function.dmx_from.value();
                let dmx_to = if *channel_function_idx >= logical_channel.channel_functions.len() - 1
                {
                    max_value(n_bytes)
                } else {
                    logical_channel.channel_functions[*channel_function_idx + 1]
                        .dmx_from
                        .value()
                        - 1
                };

                // map value (0.0..=1.0) to dmx value (dmx_from..=dmx_to)
                let dmx_value = dmx_from + ((dmx_to - dmx_from) as f32 * value) as u64;

                let value = gdtf::values::DmxValue::new(dmx_value, n_bytes, false);

                if let Some(relation_value) = Self::find_multiply_relation(
                    fixture,
                    fixture_types,
                    dmx_mode,
                    dynamic_data,
                    values,
                    channel_function,
                    grand_master,
                ) {
                    value.map(|val| multiply_dmx_value(val, relation_value))
                } else {
                    value
                }
            }
            Self::Mix { a, b, mix } => {
                if logical_channel.snap {
                    if *mix < 0.5 {
                        a._to_dmx(
                            fixture,
                            fixture_types,
                            dmx_mode,
                            dmx_channel,
                            dynamic_data,
                            values,
                            grand_master,
                        )
                    } else {
                        b._to_dmx(
                            fixture,
                            fixture_types,
                            dmx_mode,
                            dmx_channel,
                            dynamic_data,
                            values,
                            grand_master,
                        )
                    }
                } else {
                    let a = a._to_dmx(
                        fixture,
                        fixture_types,
                        dmx_mode,
                        dmx_channel,
                        dynamic_data,
                        values,
                        grand_master,
                    )?;
                    let b = b._to_dmx(
                        fixture,
                        fixture_types,
                        dmx_mode,
                        dmx_channel,
                        dynamic_data,
                        values,
                        grand_master,
                    )?;

                    let mixed = mix_dmx_value(a, b, *mix);

                    Some(mixed)
                }
            }
        };

        if logical_channel.master == gdtf::dmx_mode::LogicalChannelMaster::Grand {
            value.map(|value| multiply_dmx_value_f32(value, grand_master))
        } else {
            value
        }
    }
}
