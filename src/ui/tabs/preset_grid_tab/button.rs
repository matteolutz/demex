use egui::Response;

use super::PRESET_GRID_ELEMENT_SIZE;

pub enum PresetGridButtonConfig<'a> {
    Preset {
        id: u32,
        name: &'a str,
        top_bar_color: Option<egui::Color32>,
    },
    Empty {
        id: u32,
    },
}

impl<'a> PresetGridButtonConfig<'a> {
    pub fn id(&self) -> u32 {
        match self {
            Self::Preset { id, .. } => *id,
            Self::Empty { id } => *id,
        }
    }
}

pub fn preset_grid_button_ui(ui: &mut egui::Ui, config: PresetGridButtonConfig) -> Response {
    let (response, painter) =
        ui.allocate_painter(PRESET_GRID_ELEMENT_SIZE.into(), egui::Sense::click());

    if response.hovered() {
        ui.ctx()
            .output_mut(|out| out.cursor_icon = egui::CursorIcon::PointingHand);
    }

    painter.rect_filled(response.rect, 0.0, egui::Color32::DARK_GRAY);

    painter.text(
        response.rect.min + (2.0, 4.0).into(),
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

            painter.text(
                response.rect.center(),
                egui::Align2::CENTER_CENTER,
                name,
                egui::FontId::proportional(12.0),
                egui::Color32::WHITE,
            );
        }
        PresetGridButtonConfig::Empty { .. } => {
            painter.rect_filled(response.rect, 0.0, egui::Color32::from_black_alpha(128));
        }
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
