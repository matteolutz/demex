pub fn was_touched_slider(ui: &mut egui::Ui, value: &mut f32) -> bool {
    let slider = egui::Slider::new(value, 0.0..=1.0).vertical();

    let response = ui.add(slider);

    response.clicked() || response.dragged()
}
