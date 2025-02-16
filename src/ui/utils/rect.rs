pub fn rect_vertices(rect: &egui::Rect) -> [egui::Pos2; 4] {
    [
        rect.left_bottom(),
        rect.right_bottom(),
        rect.right_top(),
        rect.left_top(),
    ]
}
