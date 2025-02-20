use crate::fixture::presets::{preset::FixturePresetId, PresetHandler};

use super::feature_value::FixtureFeatureValue;

#[derive(Debug)]
pub enum FixtureFeatureDisplayState {
    Home,
    Preset(FixturePresetId),
    DiscreteSingleValue(f32),
    FixtureFeatureValue(FixtureFeatureValue),
}

impl FixtureFeatureDisplayState {
    pub fn is_home(&self) -> bool {
        matches!(self, Self::Home)
    }

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
            Self::DiscreteSingleValue(value) => {
                format!("{:.2}%", value * 100.0)
            }
            Self::FixtureFeatureValue(value) => value.to_string(),
        }
    }
}
