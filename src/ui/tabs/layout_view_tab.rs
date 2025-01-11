use egui::Rect;

use crate::{
    parser::nodes::fixture_selector::{
        AtomicFixtureSelector, FixtureSelector, FixtureSelectorContext,
    },
    ui::DemexUiContext,
};

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

    pub fn get_pos_and_size(&self, rect: Rect) -> (egui::Pos2, egui::Vec2) {
        let size_abs = rect.width().min(rect.height());
        let size = egui::vec2(size_abs * self.size.x, size_abs * self.size.y);

        let pos = egui::pos2(
            (rect.min.x + rect.width() * self.position.x) - size.x / 2.0,
            (rect.min.y + rect.height() * self.position.y) - size.y / 2.0,
        );

        (pos, size)
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
            fixture_layout: vec![
                FixtureLayoutEntry::new(1, egui::pos2(0.2, 0.5), egui::vec2(0.05, 0.05)),
                FixtureLayoutEntry::new(2, egui::pos2(0.8, 0.5), egui::vec2(0.05, 0.05)),
            ],
        }
    }
}

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut DemexUiContext) {
    let size = ui.available_size_before_wrap();
    let rect = Rect::from_min_size((0.0, 0.0).into(), size);

    let (response, painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());

    painter.rect_filled(rect, 0.0, egui::Color32::BLACK);

    let mut global_fixture_select_fixtures: Vec<u32> = Vec::new();
    if let Some(global_fixture_select) = &context.global_fixture_select {
        global_fixture_select_fixtures.extend(
            global_fixture_select
                .get_fixtures(
                    &context.preset_handler,
                    FixtureSelectorContext::new(&context.global_fixture_select),
                )
                .expect(""),
        );
    }

    for fixture in &context.layout_view_context.fixture_layout {
        let (pos, size) = fixture.get_pos_and_size(rect);

        painter.rect_stroke(
            Rect::from_min_size(pos, size),
            0.0,
            (
                2.0,
                if global_fixture_select_fixtures
                    .iter()
                    .any(|id| *id == fixture.fixture_id)
                {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::WHITE
                },
            ),
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
        let select_rect = Rect::from_two_pos(
            context.layout_view_context.mouse_pos.unwrap(),
            response.hover_pos().unwrap(),
        );

        let selected_fixture_ids = context
            .layout_view_context
            .fixture_layout
            .iter()
            .map(|fixture| {
                let (pos, size) = fixture.get_pos_and_size(rect);
                (fixture, Rect::from_min_size(pos, size))
            })
            .filter(|(_, fixture_rect)| select_rect.contains_rect(*fixture_rect))
            .map(|(fixture, _)| fixture.fixture_id)
            .collect::<Vec<u32>>();

        if let Some(global_fixture_select) = context.global_fixture_select.as_mut() {
            *global_fixture_select = FixtureSelector::Additive(
                AtomicFixtureSelector::FixtureIdList(selected_fixture_ids),
                Box::new(global_fixture_select.clone()),
            );
        } else {
            context.global_fixture_select = Some(FixtureSelector::Atomic(
                AtomicFixtureSelector::FixtureIdList(selected_fixture_ids),
            ));
        }

        context.layout_view_context.mouse_pos = None;
    }
}
