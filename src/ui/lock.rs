use std::path::PathBuf;

use crate::ui::constants::DEMEX_COLOR;

pub fn locked_ui(ctx: &egui::Context, locked: &mut bool, lock_image: Option<&PathBuf>) {
    eframe::egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(DEMEX_COLOR))
        .show(ctx, |ui| {
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

            ui.put(
                egui::Rect::from_min_size(egui::pos2(5.0, 5.0), egui::vec2(100.0, 10.0)),
                egui::Label::new(egui::RichText::new("Console locked").color(egui::Color32::WHITE)),
            );

            if ui.input_mut(|writer| writer.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
                *locked = false;
            }
        });
}
