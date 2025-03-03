use crate::ui::graphics::layout_projection::LayoutProjection;

#[derive(Debug, Copy, Clone)]
pub struct LayoutViewDragState {
    pub mouse_pos: egui::Pos2,
    pub projection_center: egui::Vec2,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct LayoutViewState {
    pub drag_context: Option<LayoutViewDragState>,
    pub layout_projection: LayoutProjection,
}
