use std::time;

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::{effect::FixtureChannelEffect, presets::PresetHandler};

use super::error::FixtureChannelError;

pub trait FixtureChannelValueTrait {
    fn single_default() -> Self;
    fn pair_default() -> Self;
    fn quadruple_default() -> Self;
    fn multiple_default(num_values: usize) -> Self;
    fn toggle_flag_default() -> Self;

    fn is_home(&self) -> bool;

    fn as_single(
        &self,
        preset_handler: &PresetHandler,
        fixture_id: u32,
        channel_type: u16,
    ) -> Result<f32, FixtureChannelError>;
    fn as_quadruple(
        &self,
        preset_handler: &PresetHandler,
        fixture_id: u32,
        channel_type: u16,
    ) -> Result<[f32; 4], FixtureChannelError>;
    fn as_pair(
        &self,
        preset_handler: &PresetHandler,
        fixture_id: u32,
        channel_type: u16,
    ) -> Result<[f32; 2], FixtureChannelError>;
    fn as_toggle_flag(&self, fixture_id: u32) -> Result<Option<String>, FixtureChannelError>;

    fn to_string(&self, preset_handler: &PresetHandler) -> String;

    fn with_effect_started(self, started: Option<time::Instant>) -> Self;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EguiProbe, Default)]
pub enum FixtureChannelDiscreteValue {
    Single(f32),
    Pair([f32; 2]),
    Quadruple([f32; 4]),
    Multiple(Vec<f32>),
    ToggleFlag(String),

    Effect {
        effect: FixtureChannelEffect,
    },

    #[default]
    AnyHome,
}

impl FixtureChannelDiscreteValue {
    pub fn single_default() -> Self {
        FixtureChannelDiscreteValue::Single(0.0)
    }

    pub fn pair_default() -> Self {
        FixtureChannelDiscreteValue::Pair([0.0, 0.0])
    }

    pub fn quadruple_default() -> Self {
        FixtureChannelDiscreteValue::Quadruple([0.0; 4])
    }

    pub fn multiple_default(num_values: usize) -> Self {
        FixtureChannelDiscreteValue::Multiple(vec![0.0; num_values])
    }

    pub fn toggle_flag_default() -> Self {
        unreachable!();
    }

    pub fn is_home(&self) -> bool {
        matches!(self, Self::AnyHome)
    }

    fn as_single(
        &self,
        effect_started: &Option<time::Instant>,
    ) -> Result<f32, FixtureChannelError> {
        match self {
            FixtureChannelDiscreteValue::Single(value) => Ok(*value),
            FixtureChannelDiscreteValue::AnyHome => Ok(0.0),
            FixtureChannelDiscreteValue::Effect { effect } => effect.as_single(effect_started),
            _ => Err(FixtureChannelError::FixtureChannelValueWrongVariant(
                "Single".to_owned(),
            )),
        }
    }

    fn as_quadruple(
        &self,
        effect_started: &Option<time::Instant>,
    ) -> Result<[f32; 4], FixtureChannelError> {
        match self {
            FixtureChannelDiscreteValue::Quadruple(values) => Ok(*values),
            FixtureChannelDiscreteValue::AnyHome => Ok([0.0; 4]),
            FixtureChannelDiscreteValue::Effect { effect } => effect.as_quadruple(effect_started),
            _ => Err(FixtureChannelError::FixtureChannelValueWrongVariant(
                "Quadruple".to_owned(),
            )),
        }
    }

    fn as_pair(
        &self,
        effect_started: &Option<time::Instant>,
    ) -> Result<[f32; 2], FixtureChannelError> {
        match self {
            FixtureChannelDiscreteValue::Pair(values) => Ok(*values),
            FixtureChannelDiscreteValue::AnyHome => Ok([0.0, 0.0]),
            FixtureChannelDiscreteValue::Effect { effect } => effect.as_pair(effect_started),
            _ => Err(FixtureChannelError::FixtureChannelValueWrongVariant(
                "Pair".to_owned(),
            )),
        }
    }

    fn as_toggle_flag(&self) -> Result<Option<String>, FixtureChannelError> {
        match self {
            FixtureChannelDiscreteValue::ToggleFlag(value) => Ok(Some(value.clone())),
            FixtureChannelDiscreteValue::AnyHome => Ok(None),
            _ => Err(FixtureChannelError::FixtureChannelValueWrongVariant(
                "ToggleFlag".to_owned(),
            )),
        }
    }

    fn to_string(&self, effect_started: &Option<time::Instant>) -> String {
        match self {
            FixtureChannelDiscreteValue::Single(value) => format!("{:.0}%", value * 100.0),
            FixtureChannelDiscreteValue::Pair(values) => {
                format!("{:.2}, {:.2}", values[0], values[1])
            }
            FixtureChannelDiscreteValue::Quadruple(values) => {
                format!(
                    "{:.2}, {:.2}, {:.2}, {:.2}",
                    values[0], values[1], values[2], values[3]
                )
            }
            FixtureChannelDiscreteValue::Multiple(values) => values
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            FixtureChannelDiscreteValue::ToggleFlag(value) => value.clone(),
            FixtureChannelDiscreteValue::AnyHome => "Home".to_owned(),
            FixtureChannelDiscreteValue::Effect { effect } => effect.to_string(effect_started),
        }
    }

    pub fn is_effect(&self) -> bool {
        matches!(self, Self::Effect { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EguiProbe)]
pub enum FixtureChannelValueVariant {
    Discrete(FixtureChannelDiscreteValue),
    Preset(u32),
    Mix {
        a: Box<FixtureChannelValue>,
        b: Box<FixtureChannelValue>,
        mix: f32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EguiProbe)]
pub struct FixtureChannelValue {
    variant: FixtureChannelValueVariant,

    #[serde(default, skip_serializing, skip_deserializing)]
    #[egui_probe(skip)]
    effect_started: Option<time::Instant>,
}

impl Default for FixtureChannelValue {
    fn default() -> Self {
        Self::any_home()
    }
}

impl FixtureChannelValueTrait for FixtureChannelValue {
    fn single_default() -> Self {
        FixtureChannelValue::discrete(FixtureChannelDiscreteValue::single_default())
    }

    fn pair_default() -> Self {
        FixtureChannelValue::discrete(FixtureChannelDiscreteValue::pair_default())
    }

    fn quadruple_default() -> Self {
        FixtureChannelValue::discrete(FixtureChannelDiscreteValue::quadruple_default())
    }

    fn multiple_default(num_values: usize) -> Self {
        FixtureChannelValue::discrete(FixtureChannelDiscreteValue::multiple_default(num_values))
    }

    fn toggle_flag_default() -> Self {
        FixtureChannelValue::discrete(FixtureChannelDiscreteValue::toggle_flag_default())
    }

    fn with_effect_started(mut self, started: Option<time::Instant>) -> Self {
        self.effect_started = started;
        self
    }

    fn as_single(
        &self,
        preset_handler: &PresetHandler,
        fixture_id: u32,
        channel_type: u16,
    ) -> Result<f32, FixtureChannelError> {
        match &self.variant {
            FixtureChannelValueVariant::Discrete(value) => value.as_single(&self.effect_started),
            FixtureChannelValueVariant::Mix { a, b, mix } => {
                if *mix == 0.0 {
                    return a.as_single(preset_handler, fixture_id, channel_type);
                } else if *mix == 1.0 {
                    return b.as_single(preset_handler, fixture_id, channel_type);
                }

                let a = a.as_single(preset_handler, fixture_id, channel_type)?;
                let b = b.as_single(preset_handler, fixture_id, channel_type)?;

                Ok(a * (1.0 - mix) + b * mix)
            }
            FixtureChannelValueVariant::Preset(preset_id) => {
                let preset =
                    preset_handler.get_preset_for_fixture(*preset_id, fixture_id, channel_type);

                Ok(preset
                    .map(|p| p.as_single(&self.effect_started).expect(""))
                    .unwrap_or(0.0))
            }
        }
    }

    fn as_quadruple(
        &self,
        preset_handler: &PresetHandler,
        fixture_id: u32,
        channel_type: u16,
    ) -> Result<[f32; 4], FixtureChannelError> {
        match &self.variant {
            FixtureChannelValueVariant::Discrete(value) => value.as_quadruple(&self.effect_started),
            FixtureChannelValueVariant::Preset(preset_id) => {
                let preset =
                    preset_handler.get_preset_for_fixture(*preset_id, fixture_id, channel_type);

                Ok(preset
                    .map(|p| p.as_quadruple(&self.effect_started).expect(""))
                    .unwrap_or([0.0, 0.0, 0.0, 0.0]))
            }
            FixtureChannelValueVariant::Mix { a, b, mix } => {
                if *mix == 0.0 {
                    return a.as_quadruple(preset_handler, fixture_id, channel_type);
                } else if *mix == 1.0 {
                    return b.as_quadruple(preset_handler, fixture_id, channel_type);
                }

                let a = a.as_quadruple(preset_handler, fixture_id, channel_type)?;
                let b = b.as_quadruple(preset_handler, fixture_id, channel_type)?;

                Ok([
                    a[0] * (1.0 - mix) + b[0] * mix,
                    a[1] * (1.0 - mix) + b[1] * mix,
                    a[2] * (1.0 - mix) + b[2] * mix,
                    a[3] * (1.0 - mix) + b[3] * mix,
                ])
            }
        }
    }

    fn as_pair(
        &self,
        preset_handler: &PresetHandler,
        fixture_id: u32,
        channel_type: u16,
    ) -> Result<[f32; 2], FixtureChannelError> {
        match &self.variant {
            FixtureChannelValueVariant::Discrete(value) => value.as_pair(&self.effect_started),
            FixtureChannelValueVariant::Preset(preset_id) => {
                let preset =
                    preset_handler.get_preset_for_fixture(*preset_id, fixture_id, channel_type);

                Ok(preset
                    .map(|p| p.as_pair(&self.effect_started).expect(""))
                    .unwrap_or([0.0, 0.0]))
            }
            FixtureChannelValueVariant::Mix { a, b, mix } => {
                if *mix == 0.0 {
                    return a.as_pair(preset_handler, fixture_id, channel_type);
                } else if *mix == 1.0 {
                    return b.as_pair(preset_handler, fixture_id, channel_type);
                }

                let a = a.as_pair(preset_handler, fixture_id, channel_type)?;
                let b = b.as_pair(preset_handler, fixture_id, channel_type)?;

                Ok([
                    a[0] * (1.0 - mix) + b[0] * mix,
                    a[1] * (1.0 - mix) + b[1] * mix,
                ])
            }
        }
    }

    fn as_toggle_flag(&self, fixture_id: u32) -> Result<Option<String>, FixtureChannelError> {
        match &self.variant {
            FixtureChannelValueVariant::Discrete(value) => value.as_toggle_flag(),
            FixtureChannelValueVariant::Mix { a, b, mix } => {
                if *mix > 0.5 {
                    b.as_toggle_flag(fixture_id)
                } else {
                    a.as_toggle_flag(fixture_id)
                }
            }
            unexpted => {
                println!(
                    "getting toggle flag for fixture id {}: {:?}",
                    fixture_id, unexpted
                );
                Ok(None)
            }
        }
    }

    fn is_home(&self) -> bool {
        match &self.variant {
            FixtureChannelValueVariant::Discrete(value) => value.is_home(),
            _ => false,
        }
    }

    fn to_string(&self, preset_handler: &PresetHandler) -> String {
        match &self.variant {
            FixtureChannelValueVariant::Discrete(value) => value.to_string(&self.effect_started),
            FixtureChannelValueVariant::Preset(preset_id) => {
                let preset = preset_handler.get_preset(*preset_id).map(|p| p.name());
                if let Ok(preset) = preset {
                    preset.to_owned()
                } else {
                    format!("Preset {} (deleted)", preset_id)
                }
            }
            FixtureChannelValueVariant::Mix { a, b, mix } => {
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

impl FixtureChannelValue {
    pub fn discrete(discrete_value: FixtureChannelDiscreteValue) -> Self {
        Self {
            variant: FixtureChannelValueVariant::Discrete(discrete_value),
            effect_started: None,
        }
    }

    pub fn preset(preset_id: u32) -> Self {
        Self {
            variant: FixtureChannelValueVariant::Preset(preset_id),
            effect_started: None,
        }
    }

    pub fn new(variant: FixtureChannelValueVariant) -> Self {
        Self {
            variant,
            effect_started: None,
        }
    }

    pub fn any_home() -> Self {
        Self::discrete(FixtureChannelDiscreteValue::AnyHome)
    }

    pub fn is_effect(
        &self,
        preset_handler: &PresetHandler,
        fixture_id: u32,
        channel_type: u16,
    ) -> bool {
        match &self.variant {
            FixtureChannelValueVariant::Preset(preset_id) => preset_handler
                .get_preset(*preset_id)
                .ok()
                .and_then(|p| p.value(fixture_id, channel_type))
                .map(|val| val.is_effect())
                .unwrap_or(false),
            FixtureChannelValueVariant::Discrete(discrete) => discrete.is_effect(),
            FixtureChannelValueVariant::Mix { a, b, mix } => {
                if *mix == 0.0 {
                    a.is_effect(preset_handler, fixture_id, channel_type)
                } else if *mix == 1.0 {
                    b.is_effect(preset_handler, fixture_id, channel_type)
                } else {
                    a.is_effect(preset_handler, fixture_id, channel_type)
                        || b.is_effect(preset_handler, fixture_id, channel_type)
                }
            }
        }
    }

    pub fn to_discrete(
        &self,
        fixture_id: u32,
        channel_type: u16,
        preset_handler: &PresetHandler,
    ) -> FixtureChannelDiscreteValue {
        match &self.variant {
            FixtureChannelValueVariant::Discrete(value) => value.clone(),
            FixtureChannelValueVariant::Preset(preset_id) => {
                let preset_value = preset_handler
                    .get_preset(*preset_id)
                    .map(|p| p.value(fixture_id, channel_type));

                if let Ok(Some(preset_value)) = preset_value {
                    preset_value.clone()
                } else {
                    FixtureChannelDiscreteValue::AnyHome
                }
            }
            FixtureChannelValueVariant::Mix { a, b, mix } => {
                if *mix == 0.0 {
                    return a.to_discrete(fixture_id, channel_type, preset_handler);
                } else if *mix == 1.0 {
                    return b.to_discrete(fixture_id, channel_type, preset_handler);
                }

                let a = a.to_discrete(fixture_id, channel_type, preset_handler);
                let b = b.to_discrete(fixture_id, channel_type, preset_handler);

                if a.is_home() && b.is_home() {
                    return FixtureChannelDiscreteValue::AnyHome;
                }

                todo!();
            }
        }
    }
}
