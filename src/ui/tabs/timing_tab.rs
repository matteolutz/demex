use std::time;

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut super::DemexUiContext) {
    let mut timing_handler = context.timing_handler.write();

    ui.heading("Timing");

    ui.horizontal(|ui| {
        for (speed_master_id, speed_master_value) in
            timing_handler.speed_master_values_mut().iter_mut()
        {
            ui.vertical(|ui| {
                ui.label(format!("Speed Master {}", speed_master_id));
                ui.label(speed_master_value.bpm().to_string());

                if ui
                    .button(egui::RichText::from("Tap").color(
                        if speed_master_value.display_should_blink() {
                            egui::Color32::GREEN
                        } else {
                            egui::Color32::PLACEHOLDER
                        },
                    ))
                    .clicked()
                {
                    speed_master_value.tap(time::Instant::now());
                }
            });
        }
    });
}
