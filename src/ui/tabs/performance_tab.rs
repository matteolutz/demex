pub fn ui(ui: &mut eframe::egui::Ui, context: &mut super::DemexUiContext) {
    let stats = context.stats.read();

    for (thread_name, thread_stats) in stats.stats().iter() {
        ui.vertical(|ui| {
            let heading = if let Some(thread_id) = stats.thread_id(thread_name) {
                format!("{} ({:?})", thread_name, thread_id)
            } else {
                thread_name.to_owned()
            };

            ui.heading(heading);

            ui.label(format!("dt (s): {:.5}", thread_stats.dt()));
            ui.label(format!("fps: {:.2}", 1.0 / thread_stats.dt()));

            ui.label(format!("max dt (s): {:.5}", thread_stats.max_dt()));
            ui.label(format!("min fps: {:.2}", 1.0 / thread_stats.max_dt()));

            ui.separator();
        });
    }
}
