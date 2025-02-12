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
    fn as_toggle_flag(
        &self,
        preset_handler: &PresetHandler,
        fixture_id: u32,
    ) -> Result<Option<String>, FixtureChannelError>;

    fn to_string(&self, preset_handler: &PresetHandler, channel_type: u16) -> String;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EguiProbe, Default)]
pub enum FixtureChannelDiscreteValue {
    Single(f32),
    Pair([f32; 2]),
    Quadruple([f32; 4]),
    Multiple(Vec<f32>),
    ToggleFlag(String),

    #[default]
    AnyHome,
}

impl FixtureChannelValueTrait for FixtureChannelDiscreteValue {
    fn single_default() -> Self {
        FixtureChannelDiscreteValue::Single(0.0)
    }

    fn pair_default() -> Self {
        FixtureChannelDiscreteValue::Pair([0.0, 0.0])
    }

    fn quadruple_default() -> Self {
        FixtureChannelDiscreteValue::Quadruple([0.0; 4])
    }

    fn multiple_default(num_values: usize) -> Self {
        FixtureChannelDiscreteValue::Multiple(vec![0.0; num_values])
    }

    fn toggle_flag_default() -> Self {
        unreachable!();
    }

    fn is_home(&self) -> bool {
        matches!(self, Self::AnyHome)
    }

    fn as_single(&self, _: &PresetHandler, _: u32, _: u16) -> Result<f32, FixtureChannelError> {
        match self {
            FixtureChannelDiscreteValue::Single(value) => Ok(*value),
            FixtureChannelDiscreteValue::AnyHome => Ok(0.0),
            _ => Err(FixtureChannelError::FixtureChannelValueWrongVariant(
                "Single".to_owned(),
            )),
        }
    }

    fn as_quadruple(
        &self,
        _: &PresetHandler,
        _: u32,
        _: u16,
    ) -> Result<[f32; 4], FixtureChannelError> {
        match self {
            FixtureChannelDiscreteValue::Quadruple(values) => Ok(*values),
            FixtureChannelDiscreteValue::AnyHome => Ok([0.0; 4]),
            _ => Err(FixtureChannelError::FixtureChannelValueWrongVariant(
                "Quadruple".to_owned(),
            )),
        }
    }

    fn as_pair(&self, _: &PresetHandler, _: u32, _: u16) -> Result<[f32; 2], FixtureChannelError> {
        match self {
            FixtureChannelDiscreteValue::Pair(values) => Ok(*values),
            FixtureChannelDiscreteValue::AnyHome => Ok([0.0, 0.0]),
            _ => Err(FixtureChannelError::FixtureChannelValueWrongVariant(
                "Pair".to_owned(),
            )),
        }
    }

    fn as_toggle_flag(
        &self,
        _: &PresetHandler,
        _: u32,
    ) -> Result<Option<String>, FixtureChannelError> {
        match self {
            FixtureChannelDiscreteValue::ToggleFlag(value) => Ok(Some(value.clone())),
            FixtureChannelDiscreteValue::AnyHome => Ok(None),
            _ => Err(FixtureChannelError::FixtureChannelValueWrongVariant(
                "ToggleFlag".to_owned(),
            )),
        }
    }

    fn to_string(&self, _: &PresetHandler, _: u16) -> String {
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
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EguiProbe)]
pub enum FixtureChannelValue {
    Discrete(FixtureChannelDiscreteValue),
    Preset(u32),
    Mix {
        a: Box<FixtureChannelValue>,
        b: Box<FixtureChannelValue>,
        mix: f32,
    },
    Effect {
        #[serde(default, skip_serializing, skip_deserializing)]
        #[egui_probe(skip)]
        started: Option<time::Instant>,
        effect: FixtureChannelEffect,
    },
}

impl Default for FixtureChannelValue {
    fn default() -> Self {
        Self::any_home()
    }
}

impl FixtureChannelValueTrait for FixtureChannelValue {
    fn single_default() -> Self {
        FixtureChannelValue::Discrete(FixtureChannelDiscreteValue::single_default())
    }

    fn pair_default() -> Self {
        FixtureChannelValue::Discrete(FixtureChannelDiscreteValue::pair_default())
    }

    fn quadruple_default() -> Self {
        FixtureChannelValue::Discrete(FixtureChannelDiscreteValue::quadruple_default())
    }

    fn multiple_default(num_values: usize) -> Self {
        FixtureChannelValue::Discrete(FixtureChannelDiscreteValue::multiple_default(num_values))
    }

    fn toggle_flag_default() -> Self {
        FixtureChannelValue::Discrete(FixtureChannelDiscreteValue::toggle_flag_default())
    }

    fn as_single(
        &self,
        preset_handler: &PresetHandler,
        fixture_id: u32,
        channel_type: u16,
    ) -> Result<f32, FixtureChannelError> {
        match self {
            FixtureChannelValue::Discrete(value) => {
                value.as_single(preset_handler, fixture_id, channel_type)
            }
            FixtureChannelValue::Mix { a, b, mix } => {
                let a = a.as_single(preset_handler, fixture_id, channel_type)?;
                let b = b.as_single(preset_handler, fixture_id, channel_type)?;

                Ok(a * (1.0 - mix) + b * mix)
            }
            FixtureChannelValue::Preset(preset_id) => {
                let preset =
                    preset_handler.get_preset_for_fixture(*preset_id, fixture_id, channel_type);

                Ok(preset
                    .map(|p| {
                        p.as_single(preset_handler, fixture_id, channel_type)
                            .expect("")
                    })
                    .unwrap_or(0.0))
            }
            FixtureChannelValue::Effect { started, effect } => effect.as_single(started),
        }
    }

    fn as_quadruple(
        &self,
        preset_handler: &PresetHandler,
        fixture_id: u32,
        channel_type: u16,
    ) -> Result<[f32; 4], FixtureChannelError> {
        match self {
            FixtureChannelValue::Discrete(value) => {
                value.as_quadruple(preset_handler, fixture_id, channel_type)
            }
            FixtureChannelValue::Preset(preset_id) => {
                let preset =
                    preset_handler.get_preset_for_fixture(*preset_id, fixture_id, channel_type);

                Ok(preset
                    .map(|p| {
                        p.as_quadruple(preset_handler, fixture_id, channel_type)
                            .expect("")
                    })
                    .unwrap_or([0.0, 0.0, 0.0, 0.0]))
            }
            FixtureChannelValue::Mix { a, b, mix } => {
                let a = a.as_quadruple(preset_handler, fixture_id, channel_type)?;
                let b = b.as_quadruple(preset_handler, fixture_id, channel_type)?;

                Ok([
                    a[0] * (1.0 - mix) + b[0] * mix,
                    a[1] * (1.0 - mix) + b[1] * mix,
                    a[2] * (1.0 - mix) + b[2] * mix,
                    a[3] * (1.0 - mix) + b[3] * mix,
                ])
            }
            FixtureChannelValue::Effect { started, effect } => effect.as_quadruple(started),
        }
    }

    fn as_pair(
        &self,
        preset_handler: &PresetHandler,
        fixture_id: u32,
        channel_type: u16,
    ) -> Result<[f32; 2], FixtureChannelError> {
        match self {
            FixtureChannelValue::Discrete(value) => {
                value.as_pair(preset_handler, fixture_id, channel_type)
            }
            FixtureChannelValue::Preset(preset_id) => {
                let preset =
                    preset_handler.get_preset_for_fixture(*preset_id, fixture_id, channel_type);

                Ok(preset
                    .map(|p| {
                        p.as_pair(preset_handler, fixture_id, channel_type)
                            .expect("")
                    })
                    .unwrap_or([0.0, 0.0]))
            }
            FixtureChannelValue::Mix { a, b, mix } => {
                let a = a.as_pair(preset_handler, fixture_id, channel_type)?;
                let b = b.as_pair(preset_handler, fixture_id, channel_type)?;

                Ok([
                    a[0] * (1.0 - mix) + b[0] * mix,
                    a[1] * (1.0 - mix) + b[1] * mix,
                ])
            }
            FixtureChannelValue::Effect { started, effect } => effect.as_pair(started),
        }
    }

    fn as_toggle_flag(
        &self,
        preset_handler: &PresetHandler,
        fixture_id: u32,
    ) -> Result<Option<String>, FixtureChannelError> {
        match self {
            FixtureChannelValue::Discrete(value) => {
                value.as_toggle_flag(preset_handler, fixture_id)
            }
            FixtureChannelValue::Mix { a, b, mix } => {
                let a = a.as_toggle_flag(preset_handler, fixture_id)?;
                let b = b.as_toggle_flag(preset_handler, fixture_id)?;

                if *mix > 0.5 {
                    Ok(b)
                } else {
                    Ok(a)
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
        match self {
            FixtureChannelValue::Discrete(value) => value.is_home(),
            _ => false,
        }
    }

    fn to_string(&self, preset_handler: &PresetHandler, channel_type: u16) -> String {
        match self {
            FixtureChannelValue::Discrete(value) => value.to_string(preset_handler, channel_type),
            FixtureChannelValue::Preset(preset_id) => {
                let preset = preset_handler.get_preset(*preset_id).map(|p| p.name());
                if let Ok(preset) = preset {
                    preset.to_owned()
                } else {
                    format!("Preset {} (deleted)", preset_id)
                }
            }
            FixtureChannelValue::Mix { a, b, mix } => {
                if *mix == 0.0 {
                    a.to_string(preset_handler, channel_type)
                } else if *mix == 1.0 {
                    b.to_string(preset_handler, channel_type)
                } else {
                    format!(
                        "{} * {:.2} + {} * {:.2}",
                        a.to_string(preset_handler, channel_type),
                        1.0 - mix,
                        b.to_string(preset_handler, channel_type),
                        mix
                    )
                }
            }
            FixtureChannelValue::Effect { started, effect } => effect.to_string(started),
        }
    }
}

impl FixtureChannelValue {
    pub fn any_home() -> Self {
        Self::Discrete(FixtureChannelDiscreteValue::AnyHome)
    }

    pub fn start_effect(&mut self) {
        if let Self::Effect { started, .. } = self {
            *started = Some(time::Instant::now());
        }
    }

    pub fn to_discrete(
        &self,
        fixture_id: u32,
        channel_type: u16,
        preset_handler: &PresetHandler,
    ) -> FixtureChannelDiscreteValue {
        match self {
            FixtureChannelValue::Discrete(value) => value.clone(),
            FixtureChannelValue::Preset(preset_id) => {
                let preset_value = preset_handler
                    .get_preset(*preset_id)
                    .map(|p| p.value(fixture_id, channel_type));

                if let Ok(Some(preset_value)) = preset_value {
                    preset_value.clone()
                } else {
                    FixtureChannelDiscreteValue::AnyHome
                }
            }
            FixtureChannelValue::Mix { a, b, mix } => {
                let a = a.to_discrete(fixture_id, channel_type, preset_handler);
                let b = b.to_discrete(fixture_id, channel_type, preset_handler);

                if *mix == 0.0 {
                    return a;
                } else if *mix == 1.0 {
                    return b;
                }

                if a.is_home() && b.is_home() {
                    return FixtureChannelDiscreteValue::AnyHome;
                }

                todo!();
            }
            FixtureChannelValue::Effect { .. } => FixtureChannelDiscreteValue::AnyHome,
        }
    }
}
