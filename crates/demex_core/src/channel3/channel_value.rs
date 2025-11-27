use std::time;

use serde::{Deserialize, Serialize};

use crate::{
    channel3::{
        channel_value_discrete::FixtureChannelDiscreteValue,
        channel_value_state::FixtureChannelValue3State,
    },
    fixture::{GdtfFixture, handler::FixtureTypeList},
    presets::{PresetHandler, preset::FixturePresetId},
    selection::FixtureSelection,
    timing::TimingHandler,
};

use crate::utils::serde::approx_instant;

#[derive(Debug, Clone)]
pub enum FixtureChannelValue3Discrete {
    Value(f32),
    ChannelSet(String),
}

impl FixtureChannelValue3Discrete {
    pub fn get_value(self, channel_function_idx: usize) -> FixtureChannelValue3 {
        match self {
            Self::Value(value) => FixtureChannelValue3::Discrete {
                channel_function_idx,
                value,
            },
            Self::ChannelSet(channel_set) => FixtureChannelValue3::DiscreteSet {
                channel_function_idx,
                channel_set,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FixtureChannelValue2PresetState {
    #[serde(with = "approx_instant")]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub enum FixtureChannelValue3 {
    Discrete(FixtureChannelDiscreteValue),

    Preset {
        id: FixturePresetId,

        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(feature = "ui", egui_probe(skip))]
        state: Option<FixtureChannelValue2PresetState>,
    },

    Mix {
        a: Box<Self>,
        b: Box<Self>,
        mix: f32,
    },
}

impl Default for FixtureChannelValue3 {
    fn default() -> Self {
        Self::Discrete(FixtureChannelDiscreteValue::default())
    }
}

impl PartialEq for FixtureChannelValue3 {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Home, Self::Home) => true,

            (
                Self::Preset {
                    id: preset_a,
                    state: state_a,
                },
                Self::Preset {
                    id: preset_b,
                    state: state_b,
                },
            ) => preset_a == preset_b && state_a == state_b,

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

            (
                Self::DiscreteSet {
                    channel_function_idx: idx_a,
                    channel_set: set_a,
                },
                Self::DiscreteSet {
                    channel_function_idx: idx_b,
                    channel_set: set_b,
                },
            ) => idx_a == idx_b && set_a == set_b,

            (
                Self::Mix {
                    a: a_a,
                    b: a_b,
                    mix: a_mix,
                },
                Self::Mix {
                    a: b_a,
                    b: b_b,
                    mix: b_mix,
                },
            ) => (a_mix - b_mix).abs() < f32::EPSILON && a_a == b_a && a_b == b_b,

            _ => false,
        }
    }
}

impl Eq for FixtureChannelValue3 {}

impl FixtureChannelValue3 {
    pub fn is_home(&self) -> bool {
        matches!(self, Self::Discrete(FixtureChannelDiscreteValue::Home))
    }

    pub fn should_output(
        &self,
        state: &FixtureChannelValue3State,
        preset_handler: &PresetHandler,
    ) -> bool {
        match self {
            Self::Preset { id, .. } => {
                state.was_updated
                    || preset_handler
                        .get_preset(*id)
                        .is_ok_and(|preset| preset.is_effect())
            }
            _ => state.was_updated,
        }
    }

    pub fn with_preset_state(self, preset_state: Option<FixtureChannelValue2PresetState>) -> Self {
        match self {
            Self::Discrete(_) => self,
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
        fixture_types: &FixtureTypeList,
        channel_name: &str,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> FixtureChannelDiscreteValue {
        match self {
            Self::Discrete(discrete) => discrete,
            Self::Preset { id, state } => preset_handler
                .get_preset_value_for_fixture(
                    id,
                    fixture,
                    fixture_types,
                    channel_name,
                    timing_handler,
                    state.as_ref(),
                )
                .unwrap()
                .to_discrete(
                    fixture,
                    fixture_types,
                    channel_name,
                    preset_handler,
                    timing_handler,
                ),
            Self::Mix { a, b, mix } => {
                let a = a.to_discrete(
                    fixture,
                    fixture_types,
                    channel_name,
                    preset_handler,
                    timing_handler,
                );
                let b = b.to_discrete(
                    fixture,
                    fixture_types,
                    channel_name,
                    preset_handler,
                    timing_handler,
                );
                FixtureChannelDiscreteValue::Mix {
                    a: Box::new(a),
                    b: Box::new(b),
                    mix,
                }
            }
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

    pub fn get_as_display(
        &self,
        fixture: &GdtfFixture,
        fixture_types: &FixtureTypeList,
        channel_name: &str,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> (usize, f32) {
        match self {
            Self::Discrete(discrete) => {
                discrete.get_as_display(fixture, fixture_types, channel_name)
            }
            Self::Mix { a, b, mix } => {
                let (a_idx, a_val) = a.get_as_display(
                    fixture,
                    fixture_types,
                    channel_name,
                    preset_handler,
                    timing_handler,
                );
                let (b_idx, b_val) = b.get_as_display(
                    fixture,
                    fixture_types,
                    channel_name,
                    preset_handler,
                    timing_handler,
                );

                if a_idx == b_idx {
                    (a_idx, (a_val * (1.0 - mix)) + (b_val * mix))
                } else if *mix < 0.5 {
                    (a_idx, a_val)
                } else {
                    (b_idx, b_val)
                }
            }
            Self::Preset { id, state } => preset_handler
                .get_preset_value_for_fixture(
                    *id,
                    fixture,
                    fixture_types,
                    channel_name,
                    timing_handler,
                    state.as_ref(),
                )
                .unwrap_or_default()
                .get_as_display(
                    fixture,
                    fixture_types,
                    channel_name,
                    preset_handler,
                    timing_handler,
                ),
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
            Self::DiscreteSet {
                channel_function_idx,
                channel_set,
            } => {
                format!("\"{}\" ({})", channel_set, channel_function_idx)
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
