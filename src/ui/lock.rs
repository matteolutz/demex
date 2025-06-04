use std::path::PathBuf;

use chrono::Timelike;

use crate::ui::constants::DEMEX_COLOR;

pub fn locked_ui(ctx: &egui::Context, locked: &mut bool, lock_image: Option<&PathBuf>) {
    eframe::egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(DEMEX_COLOR))
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.style_mut().spacing.item_spacing = egui::vec2(30.0, 0.0);

                    let now = chrono::Local::now();

                    ui.label(
                        egui::RichText::new(format!(
                            "{:02}:{:02}:{:02}",
                            now.hour(),
                            now.minute(),
                            now.second()
                        ))
                        .color(egui::Color32::WHITE),
                    );

                    ui.label(egui::RichText::new("Console locked").color(egui::Color32::WHITE));

                    ui.label(
                        egui::RichText::new("demex (by Matteo Lutz)").color(egui::Color32::WHITE),
                    );

                    ui.end_row();
                });
                ui.centered_and_justified(|ui| {
                    if let Some(lock_image) =
                        lock_image.and_then(|lock_image| lock_image.canonicalize().ok())
                    {
                        ui.add(
                            egui::Image::new(format!("file://{}", lock_image.to_str().unwrap()))
                                .show_loading_spinner(false),
                        );
                    } else {
                        ui.add(egui::Image::new(egui::include_image!(
                            "../../assets/LogoV1-Wide-Title.png"
                        )));
                    }
                });
            });

            /*ui.put(
                egui::Rect::from_min_size(egui::pos2(5.0, 5.0), egui::vec2(100.0, 10.0)),
                egui::Label::new(),
            );*/

            if ui.input_mut(|writer| writer.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
                *locked = false;
            }
        });
}
