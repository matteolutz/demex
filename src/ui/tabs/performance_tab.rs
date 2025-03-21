use std::thread;

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut super::DemexUiContext) {
    let fixture_handler = context.fixture_handler.read();
    let stats = context.stats.read();

    let mut num_threads = 1; // main thread

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

        num_threads += 1;
    }

    ui.vertical(|ui| {
        ui.heading("Outputs");

        for output in fixture_handler.outputs() {
            ui.horizontal(|ui| {
                ui.label(format!("{:?}", output.config()));
                ui.label(format!("(Threads: {})", output.config().num_threads()));
            });

            num_threads += output.config().num_threads();
        }
    });

    ui.vertical(|ui| {
        ui.separator();

        ui.label(format!(
            "Threads in use: >{} (/{})",
            num_threads,
            thread::available_parallelism()
                .map(|n| n.to_string())
                .unwrap_or_else(|_| "unknown".to_owned())
        ));
    });
}
