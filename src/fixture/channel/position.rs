use crate::fixture::presets::PresetHandler;

#[derive(Debug, Clone, PartialEq)]
pub enum FixturePositionValue {
    Preset(u32),
    PanTilt([f32; 2]),
}

impl FixturePositionValue {
    pub fn to_string(&self, preset_handler: &PresetHandler) -> String {
        match self {
            FixturePositionValue::Preset(preset_id) => preset_handler
                .get_position(*preset_id)
                .map_or("Preset not found", |p| p.name())
                .to_owned(),
            FixturePositionValue::PanTilt([pan, tilt]) => {
                format!("{}, {}", pan * 255.0, tilt * 255.0)
            }
        }
    }
}
