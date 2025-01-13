pub fn ui(ui: &mut eframe::egui::Ui, context: &mut super::DemexUiContext) {
    let stats = &context.stats;

    ui.label(format!("dt: {}s", stats.dt()));
    ui.label(format!("fps: {}", 1.0 / stats.dt()));
    ui.label(format!("max dt: {}s", stats.max_dt()));

    ui.label(format!("fixed update: {}s", stats.fixed_update()));
    ui.label(format!("fups: {}", 1.0 / stats.fixed_update()));
    ui.label(format!("max fixed update: {}s", stats.max_fixed_update()));
}
