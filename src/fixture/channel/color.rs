use crate::fixture::presets::PresetHandler;

#[derive(Debug, Clone, PartialEq)]
pub enum FixtureColorValue {
    Preset(u32),
    Rgbw([f32; 4]),
}

impl FixtureColorValue {
    pub fn to_string(&self, preset_handler: &PresetHandler) -> String {
        match self {
            FixtureColorValue::Preset(preset_id) => preset_handler
                .get_color(*preset_id)
                .map_or("Preset not found", |p| p.name())
                .to_owned(),
            FixtureColorValue::Rgbw([r, g, b, w]) => {
                format!("{}, {}, {}, {}", r * 255.0, g * 255.0, b * 255.0, w * 255.0)
            }
        }
    }
}
