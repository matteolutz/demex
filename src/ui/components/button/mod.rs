use egui::Ui;
use icon::DemexIcon;

pub mod icon;

pub fn icon_button(ui: &mut Ui, icon: DemexIcon) -> egui::Response {
    ui.add(egui::ImageButton::new(icon.button_image()))
}
