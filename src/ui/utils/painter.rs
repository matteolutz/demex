pub fn painter_layout_centered(
    painter: &egui::Painter,
    text: String,
    font_id: egui::FontId,
    color: egui::Color32,
    rect: egui::Rect,
) {
    let galley = painter.layout(text, font_id, color, rect.width());

    let pos = rect.center() - (galley.size() / 2.0);

    painter.galley(pos, galley, egui::Color32::PLACEHOLDER);
}
