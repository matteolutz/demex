use egui_probe::Probe;
use feature_effect::edit_feature_effect_ui;

use crate::{
    fixture::presets::preset::{FixturePreset, FixturePresetData},
    ui::{
        components::separator::padded_separator,
        window::edit::preset::keyframe_effect::edit_keyframe_effect_ui,
    },
};
pub mod feature_effect;
pub mod keyframe_effect;

pub fn edit_preset_ui(ui: &mut egui::Ui, preset: &mut FixturePreset) {
    ui.heading("Preset information");

    Probe::new(preset.name_mut()).with_header("Name").show(ui);
    Probe::new(preset.display_color_mut())
        .with_header("Display color")
        .show(ui);

    padded_separator(ui);

    let preset_id = preset.id();
    match preset.data_mut() {
        FixturePresetData::FeatureEffect { runtime } => edit_feature_effect_ui(
            ui,
            format!("Preset{}", preset_id),
            runtime,
            preset_id.feature_group,
        ),
        FixturePresetData::KeyframeEffect { runtime } => {
            edit_keyframe_effect_ui(ui, format!("Preset{}", preset_id), runtime)
        }
        _ => {}
    }
}
