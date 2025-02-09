pub fn ui(ui: &mut eframe::egui::Ui, context: &mut super::DemexUiContext) {
    let stats = context.stats.read();

    for (thread_name, thread_stats) in stats.stats().iter() {
        ui.vertical(|ui| {
            ui.heading(thread_name);

            ui.label(format!("dt: {}s", thread_stats.dt()));
            ui.label(format!("fps: {:.2}", 1.0 / thread_stats.dt()));

            ui.label(format!("max dt: {}s", thread_stats.max_dt()));
            ui.label(format!("min fps: {:.2}", 1.0 / thread_stats.max_dt()));

            ui.separator();
        });
    }
}
