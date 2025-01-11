use egui::Rect;

use crate::ui::DemexUiContext;

#[derive(Debug)]
pub struct FixtureLayoutEntry {
    fixture_id: u32,
    // relative position from 0.0 to 1.0
    position: egui::Pos2,
    // relative size from 0.0 to 1.0
    size: egui::Vec2,
}

impl FixtureLayoutEntry {
    pub fn new(fixture_id: u32, position: egui::Pos2, size: egui::Vec2) -> Self {
        Self {
            fixture_id,
            position,
            size,
        }
    }
}

pub struct LayoutViewContext {
    mouse_pos: Option<egui::Pos2>,
    fixture_layout: Vec<FixtureLayoutEntry>,
}

impl Default for LayoutViewContext {
    fn default() -> Self {
        Self {
            mouse_pos: None,
            fixture_layout: vec![FixtureLayoutEntry::new(
                1,
                egui::pos2(0.2, 0.5),
                egui::vec2(0.1, 0.1),
            )],
        }
    }
}

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut DemexUiContext) {
    let size = ui.available_size_before_wrap();
    let rect = Rect::from_min_size((0.0, 0.0).into(), size);

    let (response, painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());

    painter.rect_filled(rect, 0.0, egui::Color32::BLACK);

    for fixture in &context.layout_view_context.fixture_layout {
        let size_abs = rect.width().min(rect.height());
        let size = egui::vec2(size_abs * fixture.size.x, size_abs * fixture.size.y);

        let pos = egui::pos2(
            (rect.min.x + rect.width() * fixture.position.x) - size.x / 2.0,
            (rect.min.y + rect.height() * fixture.position.y) - size.y / 2.0,
        );

        painter.rect_stroke(
            Rect::from_min_size(pos, size),
            0.0,
            (2.0, egui::Color32::WHITE),
        );
    }

    if response.dragged() && response.hover_pos().is_some() {
        if context.layout_view_context.mouse_pos.is_none() {
            context.layout_view_context.mouse_pos = Some(response.hover_pos().unwrap());
        } else {
            painter.rect_filled(
                Rect::from_two_pos(
                    context.layout_view_context.mouse_pos.unwrap(),
                    response.hover_pos().unwrap(),
                ),
                0.0,
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 50),
            );
        }
    } else if context.layout_view_context.mouse_pos.is_some() {
        context.layout_view_context.mouse_pos = None;
    }
}
