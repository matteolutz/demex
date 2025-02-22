use std::time;

use itertools::Itertools;

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut super::DemexUiContext) {
    let mut timing_handler = context.timing_handler.write();

    ui.heading("Speed Masters");

    egui_extras::TableBuilder::new(ui)
        .columns(egui_extras::Column::auto(), 3)
        .header(20.0, |mut ui| {
            ui.col(|ui| {
                ui.strong("Id");
            });

            ui.col(|ui| {
                ui.strong("Speed");
            });

            ui.col(|ui| {
                ui.strong("Tap");
            });
        })
        .body(|mut ui| {
            for (speed_master_id, speed_master_value) in timing_handler
                .speed_master_values_mut()
                .iter_mut()
                .sorted_by_key(|(id, _)| *id)
            {
                ui.row(20.0, |mut ui| {
                    ui.col(|ui| {
                        ui.label(speed_master_id.to_string());
                    });

                    ui.col(|ui| {
                        ui.label(format!("{:.2} bpm", speed_master_value.bpm()));
                    });

                    ui.col(|ui| {
                        if ui
                            .button(egui::RichText::from("   Tap   ").color(
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
                });
            }
        });
}
