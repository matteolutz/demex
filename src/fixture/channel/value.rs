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
    ) -> Result<[f32; 4], FixtureChannelError>;
    fn as_pair(
        &self,
        preset_handler: &PresetHandler,
        fixture_id: u32,
    ) -> Result<[f32; 2], FixtureChannelError>;
    fn as_toggle_flag(
        &self,
        preset_handler: &PresetHandler,
        fixture_id: u32,
    ) -> Result<Option<String>, FixtureChannelError>;
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
        return FixtureChannelDiscreteValue::Single(0.0);
    }

    fn pair_default() -> Self {
        return FixtureChannelDiscreteValue::Pair([0.0, 0.0]);
    }

    fn quadruple_default() -> Self {
        return FixtureChannelDiscreteValue::Quadruple([0.0; 4]);
    }

    fn multiple_default(num_values: usize) -> Self {
        return FixtureChannelDiscreteValue::Multiple(vec![0.0; num_values]);
    }

    fn toggle_flag_default() -> Self {
        return FixtureChannelDiscreteValue::ToggleFlag(None);
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

    fn as_quadruple(&self, _: &PresetHandler, _: u32) -> Result<[f32; 4], FixtureChannelError> {
        match self {
            FixtureChannelDiscreteValue::Quadruple(values) => Ok(*values),
            _ => Err(FixtureChannelError::FixtureChannelValueWrongVariant(
                "Quadruple".to_owned(),
            )),
        }
    }

    fn as_pair(&self, _: &PresetHandler, _: u32) -> Result<[f32; 2], FixtureChannelError> {
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum FixtureChannelValue {
    Discrete(FixtureChannelDiscreteValue),
    Preset(u32),
}

impl FixtureChannelValueTrait for FixtureChannelValue {
    fn single_default() -> Self {
        return FixtureChannelValue::Discrete(FixtureChannelDiscreteValue::single_default());
    }

    fn pair_default() -> Self {
        return FixtureChannelValue::Discrete(FixtureChannelDiscreteValue::pair_default());
    }

    fn quadruple_default() -> Self {
        return FixtureChannelValue::Discrete(FixtureChannelDiscreteValue::quadruple_default());
    }

    fn multiple_default(num_values: usize) -> Self {
        return FixtureChannelValue::Discrete(FixtureChannelDiscreteValue::multiple_default(
            num_values,
        ));
    }

    fn toggle_flag_default() -> Self {
        return FixtureChannelValue::Discrete(FixtureChannelDiscreteValue::toggle_flag_default());
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
    ) -> Result<[f32; 4], FixtureChannelError> {
        match self {
            FixtureChannelValue::Discrete(value) => value.as_quadruple(preset_handler, fixture_id),
            _ => todo!("Preset handling for quadruple"),
        }
    }

    fn as_pair(
        &self,
        preset_handler: &PresetHandler,
        fixture_id: u32,
    ) -> Result<[f32; 2], FixtureChannelError> {
        match self {
            FixtureChannelValue::Discrete(value) => value.as_pair(preset_handler, fixture_id),
            _ => todo!("Preset handling for pair"),
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
}
