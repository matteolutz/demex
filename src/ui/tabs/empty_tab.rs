use crate::ui::context::DemexUiContext;

pub fn ui(ui: &mut egui::Ui, _: &mut DemexUiContext) {
    ui.centered_and_justified(|ui| {
        ui.heading(egui::RichText::from("Welcome to demex").color(egui::Color32::GRAY));
    });
}
