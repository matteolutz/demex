use std::time;

use itertools::Itertools;

use crate::input::event::DemexInputDeviceEvent;

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut super::DemexUiContext) {
    let mut timing_handler = context.timing_handler.write();

    ui.heading("Speed Masters");

    egui_extras::TableBuilder::new(ui)
        .columns(egui_extras::Column::auto(), 4)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
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

            ui.col(|ui| {
                ui.strong("QA's");
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
                        let response = egui_probe::Probe::new(speed_master_value.bpm_mut())
                            .with_header("")
                            .show(ui);

                        if response.changed() {
                            context.device_events.push(
                                DemexInputDeviceEvent::SpeedmasterFaderValueChanged(
                                    *speed_master_id,
                                ),
                            );
                        }

                        ui.label("bpm");
                    });

                    ui.col(|ui| {
                        if ui
                            .button(egui::RichText::from("   Tap   ").color(
                                if speed_master_value.on_beat() {
                                    ecolor::Color32::GREEN
                                } else {
                                    ecolor::Color32::PLACEHOLDER
                                },
                            ))
                            .clicked()
                        {
                            if speed_master_value.tap(time::Instant::now()) {
                                context.device_events.push(
                                    DemexInputDeviceEvent::SpeedmasterFaderValueChanged(
                                        *speed_master_id,
                                    ),
                                );
                            }
                        }
                    });

                    ui.col(|ui| {
                        if ui.button("/2").clicked() {
                            *speed_master_value.bpm_mut() /= 2.0;
                            context.device_events.push(
                                DemexInputDeviceEvent::SpeedmasterFaderValueChanged(
                                    *speed_master_id,
                                ),
                            );
                        }

                        if ui.button("*2").clicked() {
                            *speed_master_value.bpm_mut() *= 2.0;
                            context.device_events.push(
                                DemexInputDeviceEvent::SpeedmasterFaderValueChanged(
                                    *speed_master_id,
                                ),
                            );
                        }
                    });
                });
            }
        });
}
