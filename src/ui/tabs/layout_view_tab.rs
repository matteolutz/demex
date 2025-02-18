use crate::{
    fixture::{
        channel2::feature::{feature_type::FixtureFeatureType, feature_value::FixtureFeatureValue},
        layout::{FixtureLayoutDecoration, FixtureLayoutEntry, FixtureLayoutEntryType},
    },
    parser::nodes::fixture_selector::{
        AtomicFixtureSelector, FixtureSelector, FixtureSelectorContext,
    },
    ui::{
        graphics::layout_projection::LayoutProjection, utils::rect::rect_vertices, DemexUiContext,
    },
};
use egui::{
    epaint::{PathShape, PathStroke},
    Rect, Stroke,
};

impl FixtureLayoutDecoration {
    pub fn get_pos(&self, projection: &LayoutProjection, rect: &Rect) -> egui::Pos2 {
        let pos = match self {
            Self::Label {
                pos,
                text: _,
                font_size: _,
            } => pos,
        };
        projection.project(pos, rect)
    }

    pub fn draw(&self, projection: &LayoutProjection, screen: &Rect, painter: &egui::Painter) {
        let pos = self.get_pos(projection, screen);
        match self {
            Self::Label {
                pos: _,
                text,
                font_size,
            } => {
                painter.text(
                    pos,
                    egui::Align2::CENTER_CENTER,
                    text,
                    egui::FontId::proportional(font_size * projection.zoom()),
                    egui::Color32::WHITE,
                );
            }
        }
    }
}

impl FixtureLayoutEntry {
    pub fn get_pos_and_size(
        &self,
        projection: &LayoutProjection,
        rect: &Rect,
    ) -> (egui::Pos2, egui::Vec2) {
        let size = projection.scale(self.size());
        let pos = projection.project(self.position(), rect);

        (pos, size)
    }

    pub fn draw(
        &self,
        projection: &LayoutProjection,
        screen: &Rect,
        painter: &egui::Painter,
        fixture_color: egui::Color32,
        fixture_direction: Option<egui::Vec2>,
        is_selected: bool,
        is_about_to_selected: bool,
        label: impl ToString,
    ) {
        let (pos, size) = self.get_pos_and_size(projection, screen);

        let stroke_width = 0.25 * projection.zoom();
        let stroke_color = if is_selected {
            egui::Color32::DARK_GREEN
        } else if is_about_to_selected {
            egui::Color32::BLUE
        } else {
            egui::Color32::WHITE
        };

        match self.entry_type() {
            FixtureLayoutEntryType::Rect => {
                let top_left = pos - (size / 2.0);
                let rect = Rect::from_min_size(top_left, size);

                painter.rect_stroke(rect, 0.0, (stroke_width, stroke_color));

                painter.rect_filled(
                    Rect::from_min_size(
                        top_left + egui::vec2(stroke_width, stroke_width),
                        size - egui::vec2(2.0 * stroke_width, 2.0 * stroke_width),
                    ),
                    0.0,
                    fixture_color,
                );

                if let Some(fixture_direction) = fixture_direction {
                    let pos_in_rect = rect.left_top() + (fixture_direction * size);

                    for pos in rect_vertices(&rect) {
                        // draw line from pos to pos_in_rect
                        painter.line_segment(
                            [pos, pos_in_rect],
                            Stroke::new(0.25 * projection.zoom(), egui::Color32::YELLOW),
                        );
                    }

                    painter.circle_filled(
                        pos_in_rect,
                        0.5 * projection.zoom(),
                        egui::Color32::YELLOW,
                    );
                }
            }
            FixtureLayoutEntryType::Circle => {
                let radius = size.x.min(size.y) / 2.0;

                painter.circle_stroke(pos, radius, (stroke_width, stroke_color));

                painter.circle_filled(pos, radius - stroke_width, fixture_color);
            }
            FixtureLayoutEntryType::Triangle => {
                let triangle_height = size.x.min(size.y);
                let side_length = triangle_height * (2.0 / f32::sqrt(3.0));

                let points_outer = vec![
                    pos + egui::vec2(-side_length / 2.0, triangle_height / 2.0),
                    pos + egui::vec2(side_length / 2.0, triangle_height / 2.0),
                    pos + egui::vec2(0.0, -triangle_height / 2.0),
                ];

                painter.add(PathShape::convex_polygon(
                    points_outer.clone(),
                    fixture_color,
                    PathStroke::NONE,
                ));

                painter.add(PathShape::closed_line(
                    points_outer,
                    (stroke_width, stroke_color),
                ));
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

#[derive(Default)]
pub struct LayoutViewContext {
    drag_context: Option<LayoutViewDragContext>,
    layout_projection: LayoutProjection,
}

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut DemexUiContext) {
    let fixture_handler = context.fixture_handler.read();
    let preset_handler = context.preset_handler.read();
    let updatable_handler = context.updatable_handler.read();

    let fixture_layout = fixture_handler.patch().layout();
    ui.heading("Layout View");

    ui.with_layout(
        eframe::egui::Layout::left_to_right(eframe::egui::Align::LEFT),
        |ui| {
            ui.add(egui::Slider::new(
                context.layout_view_context.layout_projection.zoom_mut(),
                1.0..=20.0,
            ));

            ui.label(format!(
                "Zoom: {:.2}, Center: {:.2}",
                context.layout_view_context.layout_projection.zoom(),
                context.layout_view_context.layout_projection.center()
            ));

            if ui.button("Reset").clicked() {
                context.layout_view_context.layout_projection.reset();
            }
        },
    );

    let rect = ui.available_rect_before_wrap();
    let size = rect.size();

    ui.input(|state| {
        if state.pointer.hover_pos().is_none() || !rect.contains(state.pointer.hover_pos().unwrap())
        {
            return;
        }

        let zoom_delta = state.zoom_delta();
        if zoom_delta != 1.0 && state.pointer.hover_pos().is_some() {
            let hover_pos = state.pointer.hover_pos().unwrap();
            let center_delta: egui::Vec2 = (hover_pos - (rect.min + rect.size() / 2.0))
                / context.layout_view_context.layout_projection.zoom();

            let zoom_amount = zoom_delta - 1.0;

            *context.layout_view_context.layout_projection.zoom_mut() += zoom_amount;
            *context.layout_view_context.layout_projection.center_mut() -=
                center_delta * ((zoom_amount / 2.0).min(1.0));
        } else if state.raw_scroll_delta != egui::Vec2::ZERO {
            let offset =
                state.raw_scroll_delta / context.layout_view_context.layout_projection.zoom();
            *context.layout_view_context.layout_projection.center_mut() += offset;
        }
    });

    let (response, painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());

    if response.hovered() {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Crosshair);
    }

    if response.double_clicked() {
        context.global_fixture_select = None;
    }

    painter.rect_filled(rect, 0.0, egui::Color32::BLACK);

    let global_fixture_select_fixtures = context
        .global_fixture_select
        .as_ref()
        .and_then(|fs| {
            fs.get_fixtures(
                &preset_handler,
                FixtureSelectorContext::new(&context.global_fixture_select),
            )
            .ok()
        })
        .unwrap_or_default();

    for decoration in fixture_layout.decorations() {
        decoration.draw(
            &context.layout_view_context.layout_projection,
            &rect,
            &painter,
        );
    }

    for fixture_layout_entry in fixture_layout.fixtures() {
        let is_selected = global_fixture_select_fixtures
            .iter()
            .any(|id| *id == fixture_layout_entry.fixture_id());

        let fixture = fixture_handler
            .fixture_immut(fixture_layout_entry.fixture_id())
            .expect("todo: error handling");

        let intensity = fixture
            .feature_value(
                FixtureFeatureType::Intensity,
                &preset_handler,
                &updatable_handler,
            )
            .ok()
            .and_then(|val| match val {
                FixtureFeatureValue::Intensity { intensity } => Some(intensity),
                _ => None,
            })
            .unwrap();

        let rect_color =
            if let Ok(color) = fixture.display_color(&preset_handler, &updatable_handler) {
                egui::Color32::from_rgba_unmultiplied(
                    (color[0] * 255.0) as u8,
                    (color[1] * 255.0) as u8,
                    (color[2] * 255.0) as u8,
                    (intensity * 255.0) as u8,
                )
            } else {
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, (intensity * 255.0) as u8)
            };

        let position: Option<egui::Vec2> = fixture
            .feature_value(
                FixtureFeatureType::PositionPanTilt,
                &preset_handler,
                &updatable_handler,
            )
            .ok()
            .and_then(|val| match val {
                FixtureFeatureValue::PositionPanTilt { pan, tilt, .. } => Some((pan, tilt)),
                _ => None,
            })
            .map(Into::<egui::Vec2>::into);

        fixture_layout_entry.draw(
            &context.layout_view_context.layout_projection,
            &rect,
            &painter,
            rect_color,
            position,
            is_selected,
            false,
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

            painter.rect_stroke(
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
                egui::Stroke::new(1.0, egui::Color32::WHITE),
            );
        } else if response.dragged_by(egui::PointerButton::Middle) {
            let drag_start_world_point = context.layout_view_context.layout_projection.unproject(
                &context
                    .layout_view_context
                    .drag_context
                    .as_ref()
                    .unwrap()
                    .mouse_pos,
                &rect,
            );

            let drag_end_world_point = context
                .layout_view_context
                .layout_projection
                .unproject(response.hover_pos().as_ref().unwrap(), &rect);

            let drag_world_offset: egui::Vec2 = drag_end_world_point - drag_start_world_point;

            let world_offset: egui::Vec2 = context
                .layout_view_context
                .drag_context
                .as_ref()
                .unwrap()
                .projection_center
                + drag_world_offset;

            *context.layout_view_context.layout_projection.center_mut() = world_offset;
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

        let selected_fixture_ids = fixture_layout
            .fixtures()
            .iter()
            .map(|fixture| {
                let (pos, size) =
                    fixture.get_pos_and_size(&context.layout_view_context.layout_projection, &rect);
                (fixture, Rect::from_min_size(pos, size))
            })
            .filter(|(fixture_layout_entry, fixture_rect)| {
                select_rect.contains_rect(*fixture_rect)
                    && !global_fixture_select_fixtures.contains(&fixture_layout_entry.fixture_id())
            })
            .map(|(fixture, _)| fixture.fixture_id())
            .collect::<Vec<u32>>();

        if !selected_fixture_ids.is_empty() {
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
        }

        context.layout_view_context.drag_context = None;
    }
}
