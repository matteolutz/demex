use egui::Rect;

use crate::{
    fixture::channel::{value::FixtureChannelValueTrait, FIXTURE_CHANNEL_COLOR_ID},
    parser::nodes::fixture_selector::{
        AtomicFixtureSelector, FixtureSelector, FixtureSelectorContext,
    },
    ui::{graphics::layout_projection::LayoutProjection, DemexUiContext},
};

#[derive(Debug)]
pub enum FixtureLayoutEntryType {
    Rect,
    Circle,
}

#[derive(Debug)]
pub struct FixtureLayoutEntry {
    fixture_id: u32,
    // offset to center
    position: egui::Pos2,
    // size
    size: egui::Vec2,
    entry_type: FixtureLayoutEntryType,
}

impl FixtureLayoutEntry {
    pub fn new(
        fixture_id: u32,
        position: egui::Pos2,
        size: egui::Vec2,
        entry_type: FixtureLayoutEntryType,
    ) -> Self {
        Self {
            fixture_id,
            position,
            size,
            entry_type,
        }
    }

    pub fn get_pos_and_size(
        &self,
        projection: &LayoutProjection,
        rect: &Rect,
    ) -> (egui::Pos2, egui::Vec2) {
        let size = projection.scale(&self.size);
        let pos = projection.project(&self.position, rect);

        (pos, size)
    }

    pub fn draw(
        &self,
        projection: &LayoutProjection,
        screen: &Rect,
        painter: &egui::Painter,
        fixture_color: egui::Color32,
        is_selected: bool,
        label: impl ToString,
    ) {
        let (pos, size) = self.get_pos_and_size(projection, screen);
        let stroke_width = if is_selected { 4.0 } else { 2.0 };

        match self.entry_type {
            FixtureLayoutEntryType::Rect => {
                let top_left = pos - (size / 2.0);
                painter.rect_stroke(
                    Rect::from_min_size(top_left, size),
                    0.0,
                    (
                        stroke_width,
                        if is_selected {
                            egui::Color32::GREEN
                        } else {
                            egui::Color32::WHITE
                        },
                    ),
                );

                painter.rect_filled(
                    Rect::from_min_size(
                        top_left + egui::vec2(stroke_width, stroke_width),
                        size - egui::vec2(2.0 * stroke_width, 2.0 * stroke_width),
                    ),
                    0.0,
                    fixture_color,
                );
            }
            FixtureLayoutEntryType::Circle => {
                painter.circle_stroke(
                    pos,
                    size.x / 2.0,
                    (
                        stroke_width,
                        if is_selected {
                            egui::Color32::GREEN
                        } else {
                            egui::Color32::WHITE
                        },
                    ),
                );

                painter.circle_filled(pos, size.x / 2.0 - stroke_width, fixture_color);
            }
        }

        painter.text(
            pos + egui::vec2(0.0, size.y + 2.0),
            egui::Align2::CENTER_TOP,
            label,
            egui::FontId::proportional(2.0 * projection.zoom()),
            if is_selected {
                egui::Color32::GREEN
            } else {
                egui::Color32::WHITE
            },
        );
    }
}

pub struct LayoutViewDragContext {
    mouse_pos: egui::Pos2,
    projection_center: egui::Vec2,
}

pub struct LayoutViewContext {
    drag_context: Option<LayoutViewDragContext>,
    fixture_layout: Vec<FixtureLayoutEntry>,
    layout_projection: LayoutProjection,
}

impl Default for LayoutViewContext {
    fn default() -> Self {
        Self {
            drag_context: None,
            fixture_layout: vec![
                FixtureLayoutEntry::new(
                    3,
                    egui::pos2(-100.0, 0.0),
                    egui::vec2(2.5, 2.5),
                    FixtureLayoutEntryType::Circle,
                ),
                FixtureLayoutEntry::new(
                    4,
                    egui::pos2(-80.0, 0.0),
                    egui::vec2(2.5, 2.5),
                    FixtureLayoutEntryType::Circle,
                ),
                FixtureLayoutEntry::new(
                    5,
                    egui::pos2(-60.0, 0.0),
                    egui::vec2(2.5, 2.5),
                    FixtureLayoutEntryType::Circle,
                ),
                FixtureLayoutEntry::new(
                    1,
                    egui::pos2(-40.0, 0.0),
                    egui::vec2(5.0, 5.0),
                    FixtureLayoutEntryType::Rect,
                ),
                FixtureLayoutEntry::new(
                    6,
                    egui::pos2(-20.0, 0.0),
                    egui::vec2(2.5, 2.5),
                    FixtureLayoutEntryType::Circle,
                ),
                // CENTER
                FixtureLayoutEntry::new(
                    7,
                    egui::pos2(20.0, 0.0),
                    egui::vec2(2.0, 2.5),
                    FixtureLayoutEntryType::Circle,
                ),
                FixtureLayoutEntry::new(
                    2,
                    egui::pos2(40.0, 0.0),
                    egui::vec2(5.0, 5.0),
                    FixtureLayoutEntryType::Rect,
                ),
                FixtureLayoutEntry::new(
                    8,
                    egui::pos2(60.0, 0.0),
                    egui::vec2(2.5, 2.5),
                    FixtureLayoutEntryType::Circle,
                ),
                FixtureLayoutEntry::new(
                    9,
                    egui::pos2(80.0, 0.0),
                    egui::vec2(2.5, 2.5),
                    FixtureLayoutEntryType::Circle,
                ),
                FixtureLayoutEntry::new(
                    10,
                    egui::pos2(100.0, 0.0),
                    egui::vec2(2.5, 2.5),
                    FixtureLayoutEntryType::Circle,
                ),
            ],
            layout_projection: LayoutProjection::default(),
        }
    }
}

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut DemexUiContext) {
    ui.heading("Layout View");

    ui.with_layout(
        eframe::egui::Layout::left_to_right(eframe::egui::Align::LEFT),
        |ui| {
            ui.add(egui::Slider::new(
                context.layout_view_context.layout_projection.zoom_mut(),
                1.0..=50.0,
            ));

            ui.label(format!(
                "Zoom: {}, Center: {}",
                context.layout_view_context.layout_projection.zoom(),
                context.layout_view_context.layout_projection.center()
            ))
        },
    );

    let rect = ui.available_rect_before_wrap();
    let size = rect.size();

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

    for fixture_layout_entry in &context.layout_view_context.fixture_layout {
        let is_selected = global_fixture_select_fixtures
            .iter()
            .any(|id| *id == fixture_layout_entry.fixture_id);

        let fixture = context
            .fixture_handler
            .fixture(fixture_layout_entry.fixture_id)
            .expect("todo: error handling");

        let intensity = fixture
            .intensity()
            .expect("error handling")
            .as_single(&context.preset_handler, fixture.id())
            .expect("error handling");

        let rect_color = if let Ok(color) = fixture.color() {
            let color = color
                .as_quadruple(
                    &context.preset_handler,
                    fixture.id(),
                    FIXTURE_CHANNEL_COLOR_ID,
                )
                .unwrap();
            egui::Color32::from_rgba_unmultiplied(
                (color[0] * 255.0) as u8,
                (color[1] * 255.0) as u8,
                (color[2] * 255.0) as u8,
                (intensity * 255.0) as u8,
            )
        } else {
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, (intensity * 255.0) as u8)
        };

        fixture_layout_entry.draw(
            &context.layout_view_context.layout_projection,
            &rect,
            &painter,
            rect_color,
            is_selected,
            fixture.name(),
        );
    }

    if response.dragged() && response.hover_pos().is_some() {
        if context.layout_view_context.drag_context.is_none() {
            context.layout_view_context.drag_context = Some(LayoutViewDragContext {
                mouse_pos: response.hover_pos().unwrap(),
                projection_center: *context.layout_view_context.layout_projection.center(),
            });
        } else if response.dragged_by(egui::PointerButton::Primary) {
            painter.rect_filled(
                Rect::from_two_pos(
                    context
                        .layout_view_context
                        .drag_context
                        .as_ref()
                        .unwrap()
                        .mouse_pos,
                    response.hover_pos().unwrap(),
                ),
                0.0,
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 50),
            );
        } else if response.dragged_by(egui::PointerButton::Secondary) {
            let world_point = context
                .layout_view_context
                .layout_projection
                .unproject(response.hover_pos().as_ref().unwrap(), &rect);

            let world_offset = world_point
                - context
                    .layout_view_context
                    .drag_context
                    .as_ref()
                    .unwrap()
                    .projection_center;

            *context.layout_view_context.layout_projection.center_mut() = world_offset.to_vec2();
        }
    } else if context.layout_view_context.drag_context.is_some() && response.hover_pos().is_some() {
        let select_rect = Rect::from_two_pos(
            context
                .layout_view_context
                .drag_context
                .as_ref()
                .unwrap()
                .mouse_pos,
            response.hover_pos().unwrap(),
        );

        let selected_fixture_ids = context
            .layout_view_context
            .fixture_layout
            .iter()
            .map(|fixture| {
                let (pos, size) =
                    fixture.get_pos_and_size(&context.layout_view_context.layout_projection, &rect);
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

        context.layout_view_context.drag_context = None;
    }
}
