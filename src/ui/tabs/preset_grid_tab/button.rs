use egui::Response;

use crate::ui::{
    components::quick_menu::{QuickMenu, QuickMenuActions, QuickMenuResponse},
    utils::painter::painter_layout_centered,
};

use super::PRESET_GRID_ELEMENT_SIZE;

pub enum PresetGridButtonConfig {
    Preset {
        id: u32,
        name: String,
        top_bar_color: Option<egui::Color32>,
        display_color: Option<egui::Color32>,
    },
    Empty {
        id: u32,
    },
}

impl PresetGridButtonConfig {
    pub fn id(&self) -> u32 {
        match self {
            Self::Preset { id, .. } => *id,
            Self::Empty { id } => *id,
        }
    }
}

#[derive(Default)]
pub struct PresetGridButtonDecoration {
    pub right_top_text: Option<String>,
    pub left_bottom_text: Option<String>,
}

#[derive(Default, Clone)]
struct PresetGridButtonState {
    quick_menu_pivot: Option<egui::Pos2>,
}

pub struct PresetGridButton {
    id_source: egui::Id,
    config: PresetGridButtonConfig,
    decoration: PresetGridButtonDecoration,

    quick_menu_actions: QuickMenuActions,
}

impl PresetGridButton {
    pub fn new(
        id_source: impl Into<egui::Id>,
        config: PresetGridButtonConfig,
        decoration: PresetGridButtonDecoration,
        quick_menu_actions: QuickMenuActions,
    ) -> Self {
        Self {
            id_source: id_source.into(),
            config,
            decoration,
            quick_menu_actions,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) -> (Response, QuickMenuResponse) {
        let id = ui.make_persistent_id(self.id_source);
        let mut state = ui
            .ctx()
            .data_mut(|d| d.get_persisted::<PresetGridButtonState>(id))
            .unwrap_or_default();

        let (response, painter) = ui.allocate_painter(
            PRESET_GRID_ELEMENT_SIZE.into(),
            egui::Sense::click_and_drag(),
        );

        if response.hovered() {
            ui.ctx()
                .output_mut(|out| out.cursor_icon = egui::CursorIcon::PointingHand);
        }

        painter.rect_filled(response.rect, 0.0, egui::Color32::DARK_GRAY);

        painter.text(
            response.rect.left_top() + (2.0, 5.0).into(),
            egui::Align2::LEFT_TOP,
            self.config.id(),
            egui::FontId::proportional(12.0),
            egui::Color32::WHITE,
        );

        match self.config {
            PresetGridButtonConfig::Preset {
                name,
                top_bar_color,
                display_color,
                ..
            } => {
                if let Some(top_bar_color) = top_bar_color {
                    painter.rect_filled(
                        egui::Rect::from_min_size(
                            response.rect.min,
                            egui::vec2(response.rect.width(), 4.0),
                        ),
                        0.0,
                        top_bar_color,
                    );
                }

                painter_layout_centered(
                    &painter,
                    name,
                    egui::FontId::proportional(12.0),
                    egui::Color32::WHITE,
                    response.rect,
                );

                if let Some(display_color) = display_color {
                    painter.rect_filled(
                        egui::Rect::from_center_size(
                            response.rect.center_bottom() - egui::vec2(0.0, 10.0),
                            egui::vec2(response.rect.width() / 2.0, 10.0),
                        ),
                        2.0,
                        display_color,
                    );
                }
            }
            PresetGridButtonConfig::Empty { .. } => {
                painter.rect_filled(response.rect, 0.0, egui::Color32::from_black_alpha(128));
            }
        }

        if let Some(right_top_text) = self.decoration.right_top_text {
            painter.text(
                response.rect.right_top() + (-2.0, 5.0).into(),
                egui::Align2::RIGHT_TOP,
                right_top_text,
                egui::FontId::monospace(9.0),
                egui::Color32::LIGHT_GRAY,
            );
        }
        if let Some(left_bottom_text) = self.decoration.left_bottom_text {
            painter.text(
                response.rect.left_bottom() + (2.0, -2.0).into(),
                egui::Align2::LEFT_BOTTOM,
                left_bottom_text,
                egui::FontId::monospace(9.0),
                egui::Color32::LIGHT_GRAY,
            );
        }

        if response.hovered() {
            painter.rect_stroke(
                response.rect,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::WHITE),
            );
        }

        let quick_menu = QuickMenu::new(response.interact_rect.center(), &self.quick_menu_actions);

        let quick_menu_response =
            if response.drag_stopped() && response.interact_pointer_pos().is_some() {
                let drag_end_pos = response.interact_pointer_pos().unwrap();
                println!("drag stopped at: {:?}", drag_end_pos);

                quick_menu.interact(drag_end_pos)
            } else {
                None
            };

        if response.dragged() {
            let pivot_center = response.interact_rect.center();

            quick_menu.show(ui);
        }

        ui.ctx().data_mut(|d| d.insert_persisted(id, state));

        (response, quick_menu_response)
    }
}

pub fn preset_grid_button_ui(
    ui: &mut egui::Ui,
    config: PresetGridButtonConfig,
    decoration: PresetGridButtonDecoration,
) -> Response {
    let (response, painter) =
        ui.allocate_painter(PRESET_GRID_ELEMENT_SIZE.into(), egui::Sense::click());

    if response.hovered() {
        ui.ctx()
            .output_mut(|out| out.cursor_icon = egui::CursorIcon::PointingHand);
    }

    painter.rect_filled(response.rect, 0.0, egui::Color32::DARK_GRAY);

    painter.text(
        response.rect.left_top() + (2.0, 5.0).into(),
        egui::Align2::LEFT_TOP,
        config.id(),
        egui::FontId::proportional(12.0),
        egui::Color32::WHITE,
    );

    match config {
        PresetGridButtonConfig::Preset {
            name,
            top_bar_color,
            display_color,
            ..
        } => {
            if let Some(top_bar_color) = top_bar_color {
                painter.rect_filled(
                    egui::Rect::from_min_size(
                        response.rect.min,
                        egui::vec2(response.rect.width(), 4.0),
                    ),
                    0.0,
                    top_bar_color,
                );
            }

            painter_layout_centered(
                &painter,
                name,
                egui::FontId::proportional(12.0),
                egui::Color32::WHITE,
                response.rect,
            );

            if let Some(display_color) = display_color {
                painter.rect_filled(
                    egui::Rect::from_center_size(
                        response.rect.center_bottom() - egui::vec2(0.0, 10.0),
                        egui::vec2(response.rect.width() / 2.0, 10.0),
                    ),
                    2.0,
                    display_color,
                );
            }
        }
        PresetGridButtonConfig::Empty { .. } => {
            painter.rect_filled(response.rect, 0.0, egui::Color32::from_black_alpha(128));
        }
    }

    if let Some(right_top_text) = decoration.right_top_text {
        painter.text(
            response.rect.right_top() + (-2.0, 5.0).into(),
            egui::Align2::RIGHT_TOP,
            right_top_text,
            egui::FontId::monospace(9.0),
            egui::Color32::LIGHT_GRAY,
        );
    }
    if let Some(left_bottom_text) = decoration.left_bottom_text {
        painter.text(
            response.rect.left_bottom() + (2.0, -2.0).into(),
            egui::Align2::LEFT_BOTTOM,
            left_bottom_text,
            egui::FontId::monospace(9.0),
            egui::Color32::LIGHT_GRAY,
        );
    }

    if response.hovered() {
        painter.rect_stroke(
            response.rect,
            0.0,
            egui::Stroke::new(2.0, egui::Color32::WHITE),
        );
    }

    if response.secondary_clicked() {
        let pivot_pos = response.interact_rect.min;
    }

    response
}
