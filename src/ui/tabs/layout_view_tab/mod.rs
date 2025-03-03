use state::{LayoutViewDragState, LayoutViewState};

use crate::{
    fixture::channel2::{
        channel_type::FixtureChannelType,
        feature::{feature_type::FixtureFeatureType, feature_value::FixtureFeatureValue},
    },
    ui::{graphics::layout_projection::draw_center_of_mass, DemexUiContext},
};

mod decoration;
mod entry;
mod state;

pub struct LayoutViewComponent<'a> {
    context: &'a mut DemexUiContext,
    id_source: egui::Id,
}

impl<'a> LayoutViewComponent<'a> {
    pub fn new(context: &'a mut DemexUiContext) -> Self {
        Self {
            context,
            id_source: egui::Id::new("DemexLayoutViewComponent"),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let id = ui.make_persistent_id(self.id_source);
        let mut state = ui
            .ctx()
            .data_mut(|d| d.get_persisted::<LayoutViewState>(id))
            .unwrap_or_default();

        let fixture_handler = self.context.fixture_handler.read();
        let preset_handler = self.context.preset_handler.read();
        let updatable_handler = self.context.updatable_handler.read();
        let timing_handler = self.context.timing_handler.read();

        let fixture_layout = fixture_handler.patch().layout();
        ui.heading("Layout View");

        ui.with_layout(
            eframe::egui::Layout::left_to_right(eframe::egui::Align::LEFT),
            |ui| {
                ui.add(egui::Slider::new(
                    state.layout_projection.zoom_mut(),
                    1.0..=20.0,
                ));

                ui.label(format!(
                    "Zoom: {:.2}, Center: {:.2}",
                    state.layout_projection.zoom(),
                    state.layout_projection.center()
                ));

                if ui.button("Reset").clicked() {
                    state.layout_projection.reset();
                }
            },
        );

        let rect = ui.available_rect_before_wrap();
        let size = rect.size();

        ui.input(|reader| {
            if reader.pointer.hover_pos().is_none()
                || !rect.contains(reader.pointer.hover_pos().unwrap())
            {
                return;
            }

            let zoom_delta = reader.zoom_delta();
            if zoom_delta != 1.0 && reader.pointer.hover_pos().is_some() {
                let hover_pos = reader.pointer.hover_pos().unwrap();
                let center_delta: egui::Vec2 =
                    (hover_pos - (rect.min + rect.size() / 2.0)) / state.layout_projection.zoom();

                let zoom_amount = zoom_delta - 1.0;

                *state.layout_projection.zoom_mut() += zoom_amount;
                *state.layout_projection.center_mut() -=
                    center_delta * ((zoom_amount / 2.0).min(1.0));
            } else if reader.raw_scroll_delta != egui::Vec2::ZERO {
                let offset = reader.raw_scroll_delta / state.layout_projection.zoom();
                *state.layout_projection.center_mut() += offset;
            }
        });

        let (response, painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());
        let rect = response.rect;

        if response.hovered() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Crosshair);
        }

        if response.double_clicked() {
            self.context.global_fixture_select = None;
        }

        painter.rect_filled(rect, 0.0, egui::Color32::BLACK);

        draw_center_of_mass(
            &painter,
            state
                .layout_projection
                .project(&egui::Pos2::ZERO, &response.rect),
            2.0 * state.layout_projection.zoom(),
            egui::Color32::YELLOW,
            0.25 * state.layout_projection.zoom(),
        );

        let global_fixture_select_fixtures = self
            .context
            .global_fixture_select
            .as_ref()
            .map(|selection| selection.fixtures())
            .unwrap_or_default();

        for decoration in fixture_layout.decorations() {
            decoration.draw(&state.layout_projection, &rect, &painter);
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
                    FixtureFeatureType::SingleValue {
                        channel_type: FixtureChannelType::Intensity,
                    },
                    &preset_handler,
                    &updatable_handler,
                    &timing_handler,
                )
                .ok()
                .and_then(|val| match val {
                    FixtureFeatureValue::SingleValue { value, .. } => Some(value),
                    _ => None,
                })
                .unwrap();

            let rect_color = if let Ok(color) =
                fixture.display_color(&preset_handler, &updatable_handler, &timing_handler)
            {
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
                    &timing_handler,
                )
                .ok()
                .and_then(|val| match val {
                    FixtureFeatureValue::PositionPanTilt { pan, tilt, .. } => Some((pan, tilt)),
                    _ => None,
                })
                .map(Into::<egui::Vec2>::into);

            fixture_layout_entry.draw(
                &state.layout_projection,
                &rect,
                &painter,
                rect_color,
                position,
                is_selected,
                false,
                fixture.name(),
            );
        }

        if let Some(current_mouse_pos) = response.interact_pointer_pos() {
            if response.dragged() {
                if state.drag_context.is_none() {
                    state.drag_context = Some(LayoutViewDragState {
                        mouse_pos: current_mouse_pos,
                        projection_center: *state.layout_projection.center(),
                    });
                } else if response.dragged_by(egui::PointerButton::Primary) {
                    painter.rect_filled(
                        egui::Rect::from_two_pos(
                            state.drag_context.as_ref().unwrap().mouse_pos,
                            current_mouse_pos,
                        ),
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 50),
                    );

                    painter.rect_stroke(
                        egui::Rect::from_two_pos(
                            state.drag_context.as_ref().unwrap().mouse_pos,
                            current_mouse_pos,
                        ),
                        0.0,
                        egui::Stroke::new(1.0, egui::Color32::WHITE),
                    );
                } else if response.dragged_by(egui::PointerButton::Middle) {
                    let drag_start_world_point = state
                        .layout_projection
                        .unproject(&state.drag_context.as_ref().unwrap().mouse_pos, &rect);

                    let drag_end_world_point =
                        state.layout_projection.unproject(&current_mouse_pos, &rect);

                    let drag_world_offset: egui::Vec2 =
                        drag_end_world_point - drag_start_world_point;

                    let world_offset: egui::Vec2 =
                        state.drag_context.as_ref().unwrap().projection_center + drag_world_offset;

                    *state.layout_projection.center_mut() = world_offset;
                }
            } else if state.drag_context.is_some() {
                let select_rect = egui::Rect::from_two_pos(
                    state.drag_context.as_ref().unwrap().mouse_pos,
                    current_mouse_pos,
                );

                let selected_fixture_ids = fixture_layout
                    .fixtures()
                    .iter()
                    .map(|fixture| {
                        let (pos, size) = fixture.get_pos_and_size(&state.layout_projection, &rect);
                        (fixture, egui::Rect::from_min_size(pos, size))
                    })
                    .filter(|(fixture_layout_entry, fixture_rect)| {
                        select_rect.contains_rect(*fixture_rect)
                            && !global_fixture_select_fixtures
                                .contains(&fixture_layout_entry.fixture_id())
                    })
                    .map(|(fixture, _)| fixture.fixture_id())
                    .collect::<Vec<u32>>();

                if !selected_fixture_ids.is_empty() {
                    if let Some(global_fixture_select) = self.context.global_fixture_select.as_mut()
                    {
                        global_fixture_select.add_fixtures(&selected_fixture_ids);
                    } else {
                        self.context.global_fixture_select = Some(selected_fixture_ids.into());
                    }
                }

                state.drag_context = None;
            }
        }

        ui.ctx().data_mut(|d| d.insert_persisted(id, state));
    }
}
