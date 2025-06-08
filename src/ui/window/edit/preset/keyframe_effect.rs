use crate::fixture::keyframe_effect::effect_runtime::KeyframeEffectRuntime;

pub fn edit_keyframe_effect_ui(
    ui: &mut egui::Ui,
    _top_level_id: String,
    effect_runtime: &mut KeyframeEffectRuntime,
) {
    ui.vertical(|ui| {
        ui.heading("Keyframe Effect");

        egui_probe::Probe::new(effect_runtime.phase_mut())
            .with_header("Phase")
            .show(ui);
        egui_probe::Probe::new(effect_runtime.speed_mut())
            .with_header("Speed")
            .show(ui);

        ui.add_space(20.0);

        ui.label("TODO");
    });
}
