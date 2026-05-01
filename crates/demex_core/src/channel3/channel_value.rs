use std::time;

use serde::{Deserialize, Serialize};

use crate::{
    channel3::{
        attribute::FixtureChannel3Attribute, channel_value_discrete::FixtureChannelDiscreteValue,
        channel_value_state::FixtureChannelOutputValue, clamped_value::ClampedValue,
    },
    fixture::Fixture,
    presets::{PresetHandler, preset::FixturePresetId},
    selection::FixtureSelection,
    timing::TimingHandler,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FixtureChannelValue2PresetState {
    #[serde(default, skip_deserializing, skip_serializing)]
    started: Option<time::Instant>,

    with_selection: FixtureSelection,
}

impl FixtureChannelValue2PresetState {
    pub fn new(started: time::Instant, with_selection: FixtureSelection) -> Self {
        Self {
            started: Some(started),
            with_selection,
        }
    }

    pub fn now(selection: FixtureSelection) -> Self {
        Self {
            started: Some(time::Instant::now()),
            with_selection: selection,
        }
    }

    pub fn started(&self) -> Option<time::Instant> {
        self.started
    }

    pub fn with_started(mut self, started: Option<time::Instant>) -> Self {
        self.started = started;
        self
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
                Self::Preset {
                    id: l_id,
                    state: l_state,
                },
                Self::Preset {
                    id: r_id,
                    state: r_state,
                },
            ) => l_id == r_id && l_state == r_state,

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

    pub fn discrete(value: impl Into<ClampedValue>) -> Self {
        Self::Discrete(FixtureChannelDiscreteValue::Discrete {
            value: value.into(),
        })
    }

    pub fn discrete_set(channel_set: String) -> Self {
        Self::Discrete(FixtureChannelDiscreteValue::DiscreteSet { channel_set })
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

    pub fn with_started(self, started: Option<time::Instant>) -> Self {
        match self {
            Self::Discrete(_) => self,
            Self::Preset { id, state } => Self::Preset {
                id,
                state: state.map(|state| state.with_started(started)),
            },
            Self::Mix { a, b, mix } => Self::Mix {
                a: Box::new(a.with_started(started.clone())),
                b: Box::new(b.with_started(started)),
                mix,
            },
        }
    }

    pub fn try_as_discrete(&self) -> Option<&FixtureChannelDiscreteValue> {
        match self {
            Self::Discrete(discrete) => Some(discrete),
            _ => None,
        }
    }

    pub fn to_discrete(
        self,
        fixture: &Fixture,
        attribute: &FixtureChannel3Attribute,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> FixtureChannelDiscreteValue {
        match self {
            Self::Discrete(discrete) => discrete,
            Self::Preset { id, state } => preset_handler
                .get_preset_value_for_fixture(
                    id,
                    fixture,
                    attribute,
                    timing_handler,
                    state.as_ref(),
                )
                .unwrap_or_default()
                .to_discrete(fixture, attribute, preset_handler, timing_handler),
            Self::Mix { a, b, mix } => {
                let a = a.to_discrete(fixture, attribute, preset_handler, timing_handler);
                let b = b.to_discrete(fixture, attribute, preset_handler, timing_handler);
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
        fixture: &Fixture,
        attribute: &FixtureChannel3Attribute,
        preset_handler: &PresetHandler,
        timing_handler: &TimingHandler,
    ) -> ClampedValue {
        match self {
            Self::Discrete(discrete) => {
                if let Some(cf) = fixture.channel_function(attribute) {
                    discrete.to_clamped(cf)
                } else {
                    0.0.into()
                }
            }
            Self::Mix { a, b, mix } => {
                let a_val = a.get_as_display(fixture, attribute, preset_handler, timing_handler);
                let b_val = b.get_as_display(fixture, attribute, preset_handler, timing_handler);

                ((a_val.as_f32() * (1.0 - mix)) + (b_val.as_f32() * mix)).into()
            }
            Self::Preset { id, state } => preset_handler
                .get_preset_value_for_fixture(
                    *id,
                    fixture,
                    attribute,
                    timing_handler,
                    state.as_ref(),
                )
                .unwrap_or_default()
                .get_as_display(fixture, attribute, preset_handler, timing_handler),
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
