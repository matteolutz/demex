use serde::{Deserialize, Serialize};

use crate::{channel3::clamped_value::ClampedValue, fixture::FixtureChannelFunction};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum FixtureChannelDiscreteValue {
    #[default]
    Home,

    Discrete {
        value: ClampedValue,
    },

    DiscreteSet {
        channel_set: String,
    },

    Mix {
        a: Box<Self>,
        b: Box<Self>,
        mix: f32,
    },
}

impl FixtureChannelDiscreteValue {
    pub fn discrete(value: impl Into<ClampedValue>) -> Self {
        Self::Discrete {
            value: value.into(),
        }
    }

    pub fn discrete_set(channel_set: String) -> Self {
        Self::DiscreteSet { channel_set }
    }
}

impl PartialEq for FixtureChannelDiscreteValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Home, Self::Home) => true,
            (Self::Discrete { value: l_value }, Self::Discrete { value: r_value }) => {
                l_value == r_value
            }
            (
                Self::DiscreteSet {
                    channel_set: l_channel_set,
                },
                Self::DiscreteSet {
                    channel_set: r_channel_set,
                },
            ) => l_channel_set == r_channel_set,
            (
                Self::Mix {
                    a: l_a,
                    b: l_b,
                    mix: l_mix,
                },
                Self::Mix {
                    a: r_a,
                    b: r_b,
                    mix: r_mix,
                },
            ) => l_a == r_a && l_b == r_b && (l_mix - r_mix).abs() < f32::EPSILON,
            _ => false,
        }
    }
}

impl Eq for FixtureChannelDiscreteValue {}

impl FixtureChannelDiscreteValue {
    pub fn is_home(&self) -> bool {
        matches!(self, Self::Home)
    }

    pub fn to_clamped(&self, channel_function: &FixtureChannelFunction) -> ClampedValue {
        match self {
            &Self::Discrete { value } => value,
            Self::DiscreteSet { channel_set } => channel_function.unproject(
                channel_function
                    .set(channel_set)
                    .unwrap_or_else(|| channel_function.default()),
            ),
            Self::Home => channel_function.unprojected_default(),
            Self::Mix { a, b, mix } => {
                let a = a.to_clamped(channel_function);
                let b = b.to_clamped(channel_function);
                ((a.as_f32() * *mix) + (b.as_f32() * (1.0 - *mix))).into()
            }
        }
    }

    pub fn to_projected(
        &self,
        channel_function: &FixtureChannelFunction,
        mult: f32,
    ) -> ClampedValue {
        let val = self.to_clamped(channel_function) * mult;
        channel_function.project(val)
    }

    /*
    fn find_multiply_relation(
        patch: &Patch,
        fixture_patch: &GdtfFixturePatch,
        fixture_output_values: &HashMap<String, FixtureChannelDiscreteValue>,

        dmx_mode: &gdtf::dmx_mode::DmxMode,
        dynamic_data: &mut HashMap<String, DmxValue>,
        channel_function: &gdtf::dmx_mode::ChannelFunction,
        grand_master: f32,
    ) -> Option<gdtf::values::DmxValue> {
        let relation = dmx_mode.relations.iter().find(|rel| {
            rel.follower(dmx_mode)
                .is_some_and(|(_, _, rel_function)| rel_function == channel_function)
        });
        relation.map(|rel| {
            let relation_master = rel.master(dmx_mode).unwrap();

            let relation_master_value = fixture_output_values
                .get(relation_master.name().as_ref())
                // maybe the relation value is not yet in the output_values map
                .unwrap_or(&FixtureChannelDiscreteValue::Home);

            let value = relation_master_value
                ._to_dmx(
                    patch,
                    fixture_patch,
                    fixture_output_values,
                    dmx_mode,
                    relation_master,
                    dynamic_data,
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
        patch: &Patch,
        fixture_patch: &GdtfFixturePatch,
        fixture_output_values: &HashMap<String, FixtureChannelDiscreteValue>,
        dmx_channel: &gdtf::dmx_mode::DmxChannel,
        dynamic_data: &mut HashMap<String, DmxValue>,
        grand_master: f32,
    ) -> Option<gdtf::values::DmxValue> {
        let (_, dmx_mode) = patch.fixture_type_and_dmx_mode(fixture_patch).ok()?;

        self._to_dmx(
            patch,
            fixture_patch,
            fixture_output_values,
            dmx_mode,
            dmx_channel,
            dynamic_data,
            grand_master,
        )
    }

    fn _to_dmx(
        &self,
        patch: &Patch,
        fixture_patch: &GdtfFixturePatch,
        fixture_output_values: &HashMap<String, FixtureChannelDiscreteValue>,
        dmx_mode: &gdtf::dmx_mode::DmxMode,
        dmx_channel: &gdtf::dmx_mode::DmxChannel,
        dynamic_data: &mut HashMap<String, DmxValue>,
        grand_master: f32,
    ) -> Option<gdtf::values::DmxValue> {
        let logical_channel = &dmx_channel.logical_channels[0];

        if let Some(dynamic_value) = dynamic_data.get(dmx_channel.name().as_ref()) {
            return Some(*dynamic_value);
        }

        let value = match self {
            Self::Home => dmx_channel.initial_function().map(|(_, f)| {
                if let Some(relation_value) = Self::find_multiply_relation(
                    patch,
                    fixture_patch,
                    fixture_output_values,
                    dmx_mode,
                    dynamic_data,
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
                    patch,
                    fixture_patch,
                    fixture_output_values,
                    dmx_mode,
                    dynamic_data,
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
                    patch,
                    fixture_patch,
                    fixture_output_values,
                    dmx_mode,
                    dynamic_data,
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
                            patch,
                            fixture_patch,
                            fixture_output_values,
                            dmx_mode,
                            dmx_channel,
                            dynamic_data,
                            grand_master,
                        )
                    } else {
                        b._to_dmx(
                            patch,
                            fixture_patch,
                            fixture_output_values,
                            dmx_mode,
                            dmx_channel,
                            dynamic_data,
                            grand_master,
                        )
                    }
                } else {
                    let a = a._to_dmx(
                        patch,
                        fixture_patch,
                        fixture_output_values,
                        dmx_mode,
                        dmx_channel,
                        dynamic_data,
                        grand_master,
                    )?;
                    let b = b._to_dmx(
                        patch,
                        fixture_patch,
                        fixture_output_values,
                        dmx_mode,
                        dmx_channel,
                        dynamic_data,
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

    */
}

impl FixtureChannelDiscreteValue {
    pub fn to_string(&self) -> String {
        match self {
            Self::Home => "Home".to_owned(),
            Self::DiscreteSet { channel_set } => {
                format!("\"{}\"", channel_set)
            }
            Self::Discrete { value } => format!("{:.1}%", value.as_f32() * 100.0),
            Self::Mix { a, b, mix } => {
                if *mix == 0.0 {
                    a.to_string()
                } else if *mix == 1.0 {
                    b.to_string()
                } else {
                    format!(
                        "{} * {:.2} + {} * {:.2}",
                        a.to_string(),
                        1.0 - mix,
                        b.to_string(),
                        mix
                    )
                }
            }
        }
    }
}
