use std::{collections::HashMap, time};

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::{
    gdtf::GdtfFixture,
    handler::FixtureHandler,
    presets::{preset::FixturePresetId, PresetHandler},
    selection::FixtureSelection,
    timing::TimingHandler,
};

use super::utils::{max_value, multiply_dmx_value};

#[derive(Debug, Clone)]
pub struct FixtureChannelValue2PresetState {
    started: time::Instant,
    with_selection: FixtureSelection,
}

impl FixtureChannelValue2PresetState {
    pub fn new(started: time::Instant, with_selection: FixtureSelection) -> Self {
        Self {
            started,
            with_selection,
        }
    }

    pub fn now(selection: FixtureSelection) -> Self {
        Self {
            started: time::Instant::now(),
            with_selection: selection,
        }
    }

    pub fn started(&self) -> time::Instant {
        self.started
    }

    pub fn selection(&self) -> &FixtureSelection {
        &self.with_selection
    }
}

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

impl PartialEq for FixtureChannelValue3 {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Home, Self::Home) => true,

            // TODO: should we compare the state?
            (Self::Preset { id: preset_a, .. }, Self::Preset { id: preset_b, .. }) => {
                preset_a == preset_b
            }

            (
                Self::Discrete {
                    value: value_a,
                    channel_function_idx: idx_a,
                },
                Self::Discrete {
                    value: value_b,
                    channel_function_idx: idx_b,
                },
            ) => idx_a == idx_b && value_a == value_b,
            _ => false,
        }
    }
}

impl Eq for FixtureChannelValue3 {}

impl FixtureChannelValue3 {
    pub fn is_home(&self) -> bool {
        matches!(self, Self::Home)
    }

    pub fn with_preset_state(self, preset_state: Option<FixtureChannelValue2PresetState>) -> Self {
        match self {
            Self::Discrete { .. } | Self::Home => self,
            Self::Preset { id, state: _ } => Self::Preset {
                id,
                state: preset_state,
            },
            Self::Mix { a, b, mix } => Self::Mix {
                a: Box::new(a.with_preset_state(preset_state.clone())),
                b: Box::new(b.with_preset_state(preset_state)),
                mix,
            },
        }
    }

    pub fn to_discrete(
        self,
        fixture: &GdtfFixture,
        channel_name: &str,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> Self {
        match self {
            Self::Preset { id, state } => preset_handler
                .get_preset_value_for_fixture(
                    id,
                    fixture,
                    channel_name,
                    timing_handler,
                    state.as_ref(),
                )
                .unwrap(),
            _ => self.flatten(),
        }
    }

    pub fn flatten(self) -> Self {
        match self {
            Self::Mix { a, b, mix } => {
                if mix == 0.0 {
                    a.flatten()
                } else if mix == 1.0 {
                    b.flatten()
                } else {
                    Self::Mix { a, b, mix }
                }
            }
            val => val,
        }
    }

    pub fn get_as_discrete(
        &self,
        fixture: &GdtfFixture,
        channel_name: &str,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> (usize, f32) {
        match self {
            Self::Home => (0, 0.0),
            Self::Discrete {
                channel_function_idx,
                value,
            } => (*channel_function_idx, *value),
            Self::Mix { a, b, mix } => {
                if *mix == 0.0 {
                    a.get_as_discrete(fixture, channel_name, preset_handler, timing_handler)
                } else {
                    b.get_as_discrete(fixture, channel_name, preset_handler, timing_handler)
                }
            }
            Self::Preset { id, state } => preset_handler
                .get_preset_value_for_fixture(
                    *id,
                    fixture,
                    channel_name,
                    timing_handler,
                    state.as_ref(),
                )
                .unwrap()
                .get_as_discrete(fixture, channel_name, preset_handler, timing_handler),
        }
    }

    fn find_multiply_relation(
        dmx_mode: &gdtf::dmx_mode::DmxMode,
        values: &HashMap<String, FixtureChannelValue3>,
        channel_function: &gdtf::dmx_mode::ChannelFunction,
    ) -> Option<gdtf::values::DmxValue> {
        let relation = dmx_mode.relations.iter().find(|rel| {
            rel.follower(dmx_mode)
                .is_some_and(|(_, _, rel_function)| rel_function == channel_function)
        });
        relation.map(|rel| {
            let relation_master = rel.master(dmx_mode).unwrap();
            let relation_master_value = values.get(relation_master.name().as_ref()).unwrap();

            relation_master_value
                ._to_dmx(dmx_mode, relation_master, values)
                .unwrap()
        })
    }

    /// Converts the channel value to a DMX value (0.0..=1.0)
    pub fn to_dmx(
        &self,
        fixture_handler: &FixtureHandler,
        fixture: &GdtfFixture,
        dmx_channel: &gdtf::dmx_mode::DmxChannel,
    ) -> Option<gdtf::values::DmxValue> {
        let (_, dmx_mode) = fixture.fixture_type_and_dmx_mode(fixture_handler).ok()?;
        let values = fixture.programmer_values();

        self._to_dmx(dmx_mode, dmx_channel, values)
    }

    fn _to_dmx(
        &self,
        dmx_mode: &gdtf::dmx_mode::DmxMode,
        dmx_channel: &gdtf::dmx_mode::DmxChannel,
        values: &HashMap<String, FixtureChannelValue3>,
    ) -> Option<gdtf::values::DmxValue> {
        let logical_channel = &dmx_channel.logical_channels[0];

        match self {
            Self::Home => dmx_channel.initial_function().map(|(_, f)| {
                if let Some(relation_value) = Self::find_multiply_relation(dmx_mode, values, f) {
                    multiply_dmx_value(f.default, relation_value)
                } else {
                    f.default
                }
            }),
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

                if let Some(relation_value) =
                    Self::find_multiply_relation(dmx_mode, values, channel_function)
                {
                    value.map(|val| multiply_dmx_value(val, relation_value))
                } else {
                    value
                }
            }
            Self::Preset { .. } => Some(gdtf::values::DmxValue::default()),
            Self::Mix { .. } => Some(gdtf::values::DmxValue::default()),
        }
    }
}

impl FixtureChannelValue3 {
    pub fn to_string(&self, preset_handler: &PresetHandler) -> String {
        match self {
            Self::Home => "Home".to_owned(),
            Self::Preset { id: preset_id, .. } => {
                if let Ok(preset) = preset_handler.get_preset(*preset_id) {
                    preset.name().to_owned()
                } else {
                    format!("Preset {} (deleted)", preset_id)
                }
            }
            Self::Discrete {
                value,
                channel_function_idx,
            } => format!("{:.2} ({})", value, channel_function_idx),
            Self::Mix { a, b, mix } => {
                if *mix == 0.0 {
                    a.to_string(preset_handler)
                } else if *mix == 1.0 {
                    b.to_string(preset_handler)
                } else {
                    format!(
                        "{} * {:.2} + {} * {:.2}",
                        a.to_string(preset_handler),
                        1.0 - mix,
                        b.to_string(preset_handler),
                        mix
                    )
                }
            }
        }
    }
}
