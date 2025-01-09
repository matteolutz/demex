use crate::fixture::presets::PresetHandler;

#[derive(Debug, Clone, PartialEq)]
pub enum FixtureColorValue {
    Preset(u32, f32),
    Rgbw([f32; 4]),
}

impl FixtureColorValue {
    pub fn from_rgb(rgb: [f32; 3]) -> Self {
        FixtureColorValue::Rgbw([rgb[0], rgb[1], rgb[2], 0.0])
    }

    pub fn to_rgbw(&self, preset_handler: &PresetHandler, fixture_id: u32) -> [f32; 4] {
        match self {
            Self::Rgbw([r, g, b, _]) => [*r, *g, *b, 0.0],
            Self::Preset(preset_id, a) => preset_handler
                .get_color_for_fixture(*preset_id, fixture_id)
                .map(|v| [v[0] * a, v[1] * a, v[2] * a, v[3] * a])
                .unwrap_or([0.0, 0.0, 0.0, 0.0]),
        }
    }

    pub fn multiply(&self, a: f32) -> Self {
        match self {
            FixtureColorValue::Preset(preset_id, _) => FixtureColorValue::Preset(*preset_id, a),
            FixtureColorValue::Rgbw([r, g, b, w]) => {
                FixtureColorValue::Rgbw([*r * a, *g * a, *b * a, *w * a])
            }
        }
    }
}

impl FixtureColorValue {
    pub fn to_string(&self, preset_handler: &PresetHandler) -> String {
        match self {
            FixtureColorValue::Preset(preset_id, _) => preset_handler
                .get_color(*preset_id)
                .map_or("Preset not found", |p| p.name())
                .to_owned(),
            FixtureColorValue::Rgbw([r, g, b, w]) => {
                format!("{}, {}, {}, {}", r * 255.0, g * 255.0, b * 255.0, w * 255.0)
            }
        }
    }
}
