use effect::edit_effect_ui;

use crate::fixture::presets::preset::{FixturePreset, FixturePresetData};
pub mod effect;

pub fn edit_preset_ui(ui: &mut egui::Ui, preset: &mut FixturePreset) {
    // TODO: preset metadata

    match preset.data_mut() {
        FixturePresetData::FeatureEffect { runtime } => edit_effect_ui(ui, runtime),
        _ => {}
    }
}
