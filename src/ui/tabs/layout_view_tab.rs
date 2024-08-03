use crate::ui::DemexUiContext;

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut DemexUiContext) -> () {
    ui.label("Hello World");
}
