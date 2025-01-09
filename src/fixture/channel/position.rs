use crate::fixture::presets::PresetHandler;

#[derive(Debug, Clone, PartialEq)]
pub enum FixturePositionValue {
    Preset(u32, f32),
    PanTilt([f32; 2]),
}

impl FixturePositionValue {
    pub fn multiply(&self, a: f32) -> Self {
        match self {
            FixturePositionValue::Preset(preset_id, _) => {
                FixturePositionValue::Preset(*preset_id, a)
            }
            FixturePositionValue::PanTilt([pan, tilt]) => {
                FixturePositionValue::PanTilt([*pan * a, *tilt * a])
            }
        }
    }

    pub fn to_pan_tilt(&self, preset_handler: &PresetHandler, fixture_id: u32) -> [f32; 2] {
        match self {
            Self::PanTilt([pan, tilt]) => [*pan, *tilt],
            Self::Preset(preset_id, a) => preset_handler
                .get_position_for_fixture(*preset_id, fixture_id)
                .map(|v| [v[0] * a, v[1] * a])
                .unwrap_or([0.0, 0.0]),
        }
    }
}

impl FixturePositionValue {
    pub fn to_string(&self, preset_handler: &PresetHandler) -> String {
        match self {
            FixturePositionValue::Preset(preset_id, _) => preset_handler
                .get_position(*preset_id)
                .map_or("Preset not found", |p| p.name())
                .to_owned(),
            FixturePositionValue::PanTilt([pan, tilt]) => {
                format!("{}, {}", pan * 255.0, tilt * 255.0)
            }
        }
    }
}
