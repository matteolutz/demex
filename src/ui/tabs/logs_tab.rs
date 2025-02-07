pub fn ui(ui: &mut eframe::egui::Ui, context: &mut super::DemexUiContext) {
    egui::ScrollArea::vertical()
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for log in context.logs.iter() {
                ui.colored_label(log.color(), log.to_string());
                ui.separator();
            }
        });
}
