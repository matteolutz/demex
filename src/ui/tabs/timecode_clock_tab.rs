pub fn ui(ui: &mut eframe::egui::Ui, context: &mut super::DemexUiContext) {
    let timing_handler = context.timing_handler.read();
    let current_timecode_packet = timing_handler.current_timecode_packet();

    egui::Frame::new()
        .fill(egui::Color32::BLACK)
        .show(ui, |ui| {
            ui.add_sized(
                ui.available_size(),
                egui::Label::new(
                    egui::RichText::new(format!(
                        "{:02}:{:02}:{:02}:{:02}",
                        current_timecode_packet.hour,
                        current_timecode_packet.minute,
                        current_timecode_packet.second,
                        current_timecode_packet.frame,
                    ))
                    .extra_letter_spacing(2.0)
                    .color(egui::Color32::LIGHT_GREEN)
                    .background_color(egui::Color32::BLACK)
                    .monospace()
                    .size(80.0),
                ),
            );
        });
}
