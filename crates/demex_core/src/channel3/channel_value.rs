use std::time;

use serde::{Deserialize, Serialize};

use crate::{
    channel3::{
        channel_value_discrete::FixtureChannelDiscreteValue,
        channel_value_state::FixtureChannelOutputValue,
    },
    fixture::GdtfFixturePatch,
    patch::Patch,
    presets::{PresetHandler, preset::FixturePresetId},
    selection::FixtureSelection,
    timing::TimingHandler,
};

use crate::utils::serde::approx_instant;

#[derive(Debug, Clone)]
pub enum FixtureChannelValue3Update {
    Value(f32),
    ChannelSet(String),
}

impl FixtureChannelValue3Update {
    pub fn get_value(self, channel_function_idx: usize) -> FixtureChannelValue3 {
        match self {
            Self::Value(value) => {
                FixtureChannelValue3::Discrete(FixtureChannelDiscreteValue::Discrete {
                    channel_function_idx,
                    value,
                })
            }
            Self::ChannelSet(channel_set) => {
                FixtureChannelValue3::Discrete(FixtureChannelDiscreteValue::DiscreteSet {
                    channel_function_idx,
                    channel_set,
                })
            }
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
pub enum FixtureChannelValue3 {
    Discrete(FixtureChannelDiscreteValue),

    Preset {
        id: FixturePresetId,

        #[serde(default, skip_serializing_if = "Option::is_none")]
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
            (Self::Discrete(l), Self::Discrete(r)) => l == r,

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
    pub fn home() -> Self {
        Self::Discrete(FixtureChannelDiscreteValue::Home)
    }

    pub fn is_home(&self) -> bool {
        matches!(self, Self::Discrete(FixtureChannelDiscreteValue::Home))
    }

    pub fn should_output(
        &self,
        state: &FixtureChannelOutputValue,
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
        patch: &Patch,
        fixture: &GdtfFixturePatch,
        channel_name: &str,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> FixtureChannelDiscreteValue {
        match self {
            Self::Discrete(discrete) => discrete,
            Self::Preset { id, state } => preset_handler
                .get_preset_value_for_fixture(
                    id,
                    patch,
                    fixture,
                    channel_name,
                    timing_handler,
                    state.as_ref(),
                )
                .unwrap()
                .to_discrete(patch, fixture, channel_name, preset_handler, timing_handler),
            Self::Mix { a, b, mix } => {
                let a = a.to_discrete(patch, fixture, channel_name, preset_handler, timing_handler);
                let b = b.to_discrete(patch, fixture, channel_name, preset_handler, timing_handler);
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
        patch: &Patch,
        fixture: &GdtfFixturePatch,
        channel_name: &str,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> (usize, f32) {
        match self {
            Self::Discrete(discrete) => discrete.get_as_display(patch, fixture, channel_name),
            Self::Mix { a, b, mix } => {
                let (a_idx, a_val) =
                    a.get_as_display(patch, fixture, channel_name, preset_handler, timing_handler);
                let (b_idx, b_val) =
                    b.get_as_display(patch, fixture, channel_name, preset_handler, timing_handler);

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
                    patch,
                    fixture,
                    channel_name,
                    timing_handler,
                    state.as_ref(),
                )
                .unwrap_or_default()
                .get_as_display(patch, fixture, channel_name, preset_handler, timing_handler),
        }
    }
}

impl FixtureChannelValue3 {
    pub fn to_opaque_string(&self) -> String {
        match self {
            Self::Preset { id: preset_id, .. } => {
                format!("Preset {}", preset_id)
            }
            Self::Discrete(discrete) => discrete.to_string(),
            Self::Mix { a, b, mix } => {
                if *mix == 0.0 {
                    a.to_opaque_string()
                } else if *mix == 1.0 {
                    b.to_opaque_string()
                } else {
                    format!(
                        "{} * {:.2} + {} * {:.2}",
                        a.to_opaque_string(),
                        1.0 - mix,
                        b.to_opaque_string(),
                        mix
                    )
                }
            }
        }
    }

    pub fn to_string(&self, preset_handler: &PresetHandler) -> String {
        match self {
            Self::Preset { id: preset_id, .. } => {
                if let Ok(preset) = preset_handler.get_preset(*preset_id) {
                    preset.name().to_owned()
                } else {
                    format!("Preset {} (deleted)", preset_id)
                }
            }
            Self::Discrete(discrete) => discrete.to_string(),
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
