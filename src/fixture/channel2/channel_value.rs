use std::hash::Hash;

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::presets::PresetHandler;

use super::{channel_type::FixtureChannelType, error::FixtureChannelError2};

#[derive(Debug, Serialize, Deserialize, Clone, Default, EguiProbe)]
pub enum FixtureChannelValue2 {
    #[default]
    Home,

    Preset(u32),
    Discrete(u8),
    Mix {
        a: Box<FixtureChannelValue2>,
        b: Box<FixtureChannelValue2>,
        mix: f32,
    },
}

impl PartialEq for FixtureChannelValue2 {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Home, Self::Home) => true,
            (Self::Preset(preset_a), Self::Preset(preset_b)) => preset_a == preset_b,
            (Self::Discrete(value_a), Self::Discrete(value_b)) => value_a == value_b,
            _ => false,
        }
    }
}

impl Eq for FixtureChannelValue2 {}

impl Hash for FixtureChannelValue2 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Home => 0.hash(state),
            Self::Preset(preset_id) => {
                1.hash(state);
                preset_id.hash(state);
            }
            Self::Discrete(value) => {
                2.hash(state);
                value.hash(state);
            }
            Self::Mix { .. } => {
                3.hash(state);
            }
        }
    }
}

impl FixtureChannelValue2 {
    pub fn discrete_f32(val: f32) -> Self {
        Self::Discrete((val * 255.0) as u8)
    }

    pub fn is_home(&self) -> bool {
        matches!(self, Self::Home)
    }

    pub fn to_discrete_value(
        &self,
        fixture_id: u32,
        channel_type: FixtureChannelType,
        preset_handler: &PresetHandler,
    ) -> Result<u8, FixtureChannelError2> {
        match self {
            Self::Home => Ok(0),
            Self::Discrete(value) => Ok(*value),
            Self::Preset(preset_id) => preset_handler
                .get_preset_value_for_fixture(*preset_id, fixture_id, channel_type)
                .ok_or(FixtureChannelError2::PresetNotFound(*preset_id)),
            Self::Mix { a, b, mix } => {
                if *mix == 0.0 {
                    a.to_discrete_value(fixture_id, channel_type, preset_handler)
                } else if *mix == 1.0 {
                    b.to_discrete_value(fixture_id, channel_type, preset_handler)
                } else {
                    let a = a.to_discrete_value(fixture_id, channel_type, preset_handler)?;
                    let b = b.to_discrete_value(fixture_id, channel_type, preset_handler)?;

                    Ok((a as f32 * (1.0 - mix) + b as f32 * mix) as u8)
                }
            }
        }
    }
}

impl FixtureChannelValue2 {
    pub fn to_string(&self, preset_handler: &PresetHandler) -> String {
        match self {
            Self::Home => "Home".to_owned(),
            Self::Preset(preset_id) => {
                if let Ok(preset) = preset_handler.get_preset(*preset_id) {
                    preset.name().to_owned()
                } else {
                    format!("Preset {} (deleted)", preset_id)
                }
            }
            Self::Discrete(value) => value.to_string(),
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
