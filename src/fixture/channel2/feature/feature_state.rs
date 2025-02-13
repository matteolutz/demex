use crate::fixture::{channel2::channel_value::FixtureChannelValue2, presets::PresetHandler};

#[derive(Debug)]
pub enum FixtureFeatureDisplayState {
    Home,
    Preset(u32),
    DiscreteSingleValue(f32),
    FixtureChannelValue(FixtureChannelValue2),
}

impl FixtureFeatureDisplayState {
    pub fn is_home(&self) -> bool {
        match self {
            Self::Home => true,
            Self::FixtureChannelValue(value) => value.is_home(),
            _ => false,
        }
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
            Self::FixtureChannelValue(value) => value.to_string(preset_handler),
        }
    }
}
