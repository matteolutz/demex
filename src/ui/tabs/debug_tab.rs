pub fn ui(ui: &mut eframe::egui::Ui, context: &mut super::DemexUiContext) {
    let stats = context.stats.read();

    ui.label(format!("dt: {}s", stats.ui_dt));
    ui.label(format!("fps: {}", 1.0 / stats.ui_dt));
    ui.label(format!("max dt: {}s", stats.ui_max_dt));

    ui.label(format!("fixed update: {}s", stats.fixed_update_dt));
    ui.label(format!("fups: {}", 1.0 / stats.fixed_update_dt));
    ui.label(format!("max fixed update: {}s", stats.fixed_update_max_dt));
}
