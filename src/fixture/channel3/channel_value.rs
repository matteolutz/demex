use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::{
    channel2::channel_value::FixtureChannelValue2PresetState, presets::preset::FixturePresetId,
};

use super::utils::multiply_dmx_value;

#[derive(Debug, Serialize, Deserialize, Clone, Default, EguiProbe)]
pub enum FixtureChannelValue3 {
    #[default]
    Home,

    Preset {
        id: FixturePresetId,

        #[serde(default, skip_serializing, skip_deserializing)]
        #[egui_probe(skip)]
        state: Option<FixtureChannelValue2PresetState>,
    },

    Discrete {
        channel_function_idx: usize,
        value: f32,
    },

    Mix {
        a: Box<Self>,
        b: Box<Self>,
        mix: f32,
    },
}

impl FixtureChannelValue3 {
    /// Converts the channel value to a DMX value (0.0..=1.0)
    pub fn to_dmx(
        &self,
        dmx_mode: &gdtf::dmx_mode::DmxMode,
        dmx_channel: &gdtf::dmx_mode::DmxChannel,
        values: &[FixtureChannelValue3],
    ) -> Option<gdtf::values::DmxValue> {
        log::info!("got called on {}", dmx_channel.name().as_ref());
        let logical_channel = &dmx_channel.logical_channels[0];

        match self {
            Self::Home => dmx_channel.initial_function().map(|(_, f)| {
                let relation = dmx_mode.relations.iter().find(|rel| {
                    rel.follower(dmx_mode)
                        .is_some_and(|(_, _, rel_function)| rel_function == f)
                });

                if let Some(relation) = relation {
                    log::info!("found relation: {:?}", relation);
                    let relation_master = relation.master(dmx_mode).unwrap();
                    let relation_master_value = &values[dmx_mode
                        .dmx_channels
                        .iter()
                        .position(|ch| ch == relation_master)
                        .unwrap()];

                    let relation_master_dmx_value = relation_master_value
                        .to_dmx(dmx_mode, dmx_channel, values)
                        .unwrap();

                    multiply_dmx_value(f.default, relation_master_dmx_value)
                } else {
                    f.default
                }
            }),
            Self::Discrete {
                channel_function_idx,
                value,
            } => {
                let channel_function = &logical_channel.channel_functions[*channel_function_idx];

                let dmx_from = channel_function.dmx_from.value();
                let dmx_to = if *channel_function_idx >= logical_channel.channel_functions.len() - 1
                {
                    let max_value = (2u64).pow(channel_function.dmx_from.bytes().get() as u32);
                    (max_value - 1) as u64
                } else {
                    logical_channel.channel_functions[*channel_function_idx + 1]
                        .dmx_from
                        .value()
                        - 1
                };

                // map value (0.0..=1.0) to dmx value (dmx_from..=dmx_to)
                let dmx_value = dmx_from + ((dmx_to - dmx_from) as f32 * value) as u64;

                gdtf::values::DmxValue::new(dmx_value, channel_function.dmx_from.bytes(), false)
            }
            Self::Preset { .. } => Some(gdtf::values::DmxValue::default()),
            Self::Mix { .. } => Some(gdtf::values::DmxValue::default()),
        }
    }
}
