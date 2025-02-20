use egui::Response;

use crate::ui::utils::painter::painter_layout_centered;

use super::PRESET_GRID_ELEMENT_SIZE;

pub enum PresetGridButtonConfig {
    Preset {
        id: u32,
        name: String,
        top_bar_color: Option<egui::Color32>,
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

    response
}
