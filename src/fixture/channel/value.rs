use crate::fixture::presets::PresetHandler;

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

#[derive(Debug, Clone, PartialEq)]
pub enum FixtureChannelDiscreteValue {
    Single(f32),
    Pair([f32; 2]),
    Quadruple([f32; 4]),
    Multiple(Vec<f32>),
    ToggleFlag(Option<String>),
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
        FixtureChannelDiscreteValue::ToggleFlag(None)
    }

    fn is_home(&self) -> bool {
        match self {
            FixtureChannelDiscreteValue::Single(value) => *value == 0.0,
            FixtureChannelDiscreteValue::Pair(values) => values.iter().all(|v| *v == 0.0),
            FixtureChannelDiscreteValue::Quadruple(values) => values.iter().all(|v| *v == 0.0),
            FixtureChannelDiscreteValue::Multiple(values) => values.iter().all(|v| *v == 0.0),
            FixtureChannelDiscreteValue::ToggleFlag(value) => value.is_none(),
        }
    }

    fn as_single(&self, _: &PresetHandler, _: u32) -> Result<f32, FixtureChannelError> {
        match self {
            FixtureChannelDiscreteValue::Single(value) => Ok(*value),
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
            _ => Err(FixtureChannelError::FixtureChannelValueWrongVariant(
                "Quadruple".to_owned(),
            )),
        }
    }

    fn as_pair(&self, _: &PresetHandler, _: u32, _: u16) -> Result<[f32; 2], FixtureChannelError> {
        match self {
            FixtureChannelDiscreteValue::Pair(values) => Ok(*values),
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
            FixtureChannelDiscreteValue::ToggleFlag(value) => Ok(value.clone()),
            _ => Err(FixtureChannelError::FixtureChannelValueWrongVariant(
                "ToggleFlag".to_owned(),
            )),
        }
    }

    fn to_string(&self, _: &PresetHandler, _: u16) -> String {
        match self {
            FixtureChannelDiscreteValue::Single(value) => format!("{}%", value * 100.0),
            FixtureChannelDiscreteValue::Pair(values) => {
                format!("{}, {}", values[0], values[1])
            }
            FixtureChannelDiscreteValue::Quadruple(values) => {
                format!("{}, {}, {}, {}", values[0], values[1], values[2], values[3])
            }
            FixtureChannelDiscreteValue::Multiple(values) => values
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            FixtureChannelDiscreteValue::ToggleFlag(value) => match value {
                Some(flag) => flag.clone(),
                None => "None".to_owned(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FixtureChannelValue {
    Discrete(FixtureChannelDiscreteValue),
    Preset(u32),
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
    ) -> Result<f32, FixtureChannelError> {
        match self {
            FixtureChannelValue::Discrete(value) => value.as_single(preset_handler, fixture_id),
            _ => todo!("Preset handling for single"),
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
                    preset_handler.get_preset_for_fixture(*preset_id, channel_type, fixture_id);

                Ok(preset
                    .map(|p| {
                        p.as_quadruple(preset_handler, fixture_id, channel_type)
                            .expect("")
                    })
                    .unwrap_or([0.0, 0.0, 0.0, 0.0]))
            }
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
                    preset_handler.get_preset_for_fixture(*preset_id, channel_type, fixture_id);

                Ok(preset
                    .map(|p| {
                        p.as_pair(preset_handler, fixture_id, channel_type)
                            .expect("")
                    })
                    .unwrap_or([0.0, 0.0]))
            }
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
            _ => todo!("Preset handling for toggle flag"),
        }
    }

    fn is_home(&self) -> bool {
        match self {
            FixtureChannelValue::Discrete(value) => value.is_home(),
            FixtureChannelValue::Preset(_) => false,
        }
    }

    fn to_string(&self, preset_handler: &PresetHandler, channel_type: u16) -> String {
        match self {
            FixtureChannelValue::Discrete(value) => value.to_string(preset_handler, channel_type),
            FixtureChannelValue::Preset(preset_id) => {
                let preset = preset_handler
                    .get_preset(*preset_id, channel_type)
                    .map(|p| p.name());
                if let Ok(preset) = preset {
                    preset.to_owned()
                } else {
                    format!("Preset {}.{}", channel_type, preset_id)
                }
            }
        }
    }
}

impl FixtureChannelValue {
    pub fn to_discrete(&self) -> FixtureChannelDiscreteValue {
        match self {
            FixtureChannelValue::Discrete(value) => value.clone(),
            FixtureChannelValue::Preset(_) => todo!("Preset handling for to_discrete"),
        }
    }
}
