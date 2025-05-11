use crate::{
    fixture::effect::feature::runtime::FeatureEffectRuntime,
    ui::components::wave_editor::wave_editor_ui,
};

pub fn edit_effect_ui(ui: &mut egui::Ui, effect_runtime: &mut FeatureEffectRuntime) {
    ui.vertical(|ui| {
        ui.heading("Effect");
        for (idx, part) in effect_runtime
            .effect_mut()
            .parts_mut()
            .iter_mut()
            .enumerate()
        {
            ui.vertical(|ui| {
                ui.label(format!("Part {}", idx + 1));
                wave_editor_ui(ui, part.wave_mut());
            });
        }
    });
    ui.label(format!("Effect: {:?}", effect_runtime.effect()));
}
