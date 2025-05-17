use egui::Response;

use crate::ui::{
    components::quick_menu::{QuickMenu, QuickMenuAction, QuickMenuActions, QuickMenuResponse},
    utils::painter::painter_layout_centered,
};

use super::PRESET_GRID_ELEMENT_SIZE;

pub enum PresetGridButtonConfig {
    Preset {
        id: u32,
        name: String,
        top_bar_color: Option<ecolor::Color32>,
        display_color: Option<ecolor::Color32>,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PresetGridButtonQuickMenuActions {
    Edit,
    Insert,
    Default,

    New,

    Custom(&'static str),
}

impl std::fmt::Display for PresetGridButtonQuickMenuActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PresetGridButtonQuickMenuActions::Edit => write!(f, "Edit"),
            PresetGridButtonQuickMenuActions::Insert => write!(f, "Insert"),
            PresetGridButtonQuickMenuActions::Default => write!(f, "Default"),
            PresetGridButtonQuickMenuActions::New => write!(f, "New"),
            PresetGridButtonQuickMenuActions::Custom(custom) => write!(f, "{}", custom),
        }
    }
}

impl From<PresetGridButtonQuickMenuActions> for QuickMenuAction<PresetGridButtonQuickMenuActions> {
    fn from(value: PresetGridButtonQuickMenuActions) -> Self {
        (value, value.to_string()).into()
    }
}

pub struct PresetGridButton {
    config: PresetGridButtonConfig,
    decoration: PresetGridButtonDecoration,

    quick_menu_actions: QuickMenuActions<PresetGridButtonQuickMenuActions>,
}

impl PresetGridButton {
    pub fn new(
        config: PresetGridButtonConfig,
        decoration: PresetGridButtonDecoration,
        quick_menu_actions: Option<Vec<PresetGridButtonQuickMenuActions>>,
        quick_menu_actions_empty: Option<Vec<PresetGridButtonQuickMenuActions>>,
    ) -> Self {
        Self {
            quick_menu_actions: match config {
                PresetGridButtonConfig::Empty { .. } => {
                    let mut actions =
                        QuickMenuActions::default().top_left(PresetGridButtonQuickMenuActions::New);

                    if let Some(quick_menu_actions_empty) = quick_menu_actions_empty {
                        actions = actions.with_vec(quick_menu_actions_empty.into());
                    }

                    actions
                }
                PresetGridButtonConfig::Preset { .. } => {
                    let mut actions = QuickMenuActions::default()
                        .top_left(PresetGridButtonQuickMenuActions::Insert)
                        .top_center(PresetGridButtonQuickMenuActions::Edit)
                        .top_right(PresetGridButtonQuickMenuActions::Default);

                    if let Some(quick_menu_actions) = quick_menu_actions {
                        actions = actions.with_vec(quick_menu_actions.into());
                    }

                    actions
                }
            },
            config,
            decoration,
        }
    }

    pub fn show(
        self,
        ui: &mut egui::Ui,
    ) -> (
        Response,
        QuickMenuResponse<PresetGridButtonQuickMenuActions>,
    ) {
        let (response, painter) = ui.allocate_painter(
            PRESET_GRID_ELEMENT_SIZE.into(),
            egui::Sense::click_and_drag(),
        );

        if response.hovered() {
            ui.ctx()
                .output_mut(|out| out.cursor_icon = egui::CursorIcon::PointingHand);
        }

        painter.rect_filled(response.rect, 0.0, ecolor::Color32::DARK_GRAY);

        painter.text(
            response.rect.left_top() + (2.0, 5.0).into(),
            egui::Align2::LEFT_TOP,
            self.config.id(),
            egui::FontId::proportional(12.0),
            ecolor::Color32::WHITE,
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
                    ecolor::Color32::WHITE,
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
                painter.rect_filled(response.rect, 0.0, ecolor::Color32::from_black_alpha(128));
            }
        }

        if let Some(right_top_text) = self.decoration.right_top_text {
            painter.text(
                response.rect.right_top() + (-2.0, 5.0).into(),
                egui::Align2::RIGHT_TOP,
                right_top_text,
                egui::FontId::monospace(9.0),
                ecolor::Color32::LIGHT_GRAY,
            );
        }
        if let Some(left_bottom_text) = self.decoration.left_bottom_text {
            painter.text(
                response.rect.left_bottom() + (2.0, -2.0).into(),
                egui::Align2::LEFT_BOTTOM,
                left_bottom_text,
                egui::FontId::monospace(9.0),
                ecolor::Color32::LIGHT_GRAY,
            );
        }

        if response.hovered() {
            painter.rect_stroke(
                response.rect,
                0.0,
                egui::Stroke::new(2.0, ecolor::Color32::WHITE),
                egui::StrokeKind::Middle,
            );
        }

        let quick_menu = QuickMenu::new(response.interact_rect.center(), &self.quick_menu_actions);

        let quick_menu_response = response
            .drag_stopped()
            .then(|| response.interact_pointer_pos())
            .flatten()
            .and_then(|pos| quick_menu.interact(pos));

        if response.dragged()
            && !ui.input(|reader| {
                reader
                    .multi_touch()
                    .is_some_and(|touch| touch.num_touches == 2)
            })
        {
            quick_menu.show(ui);
        }

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

    painter.rect_filled(response.rect, 0.0, ecolor::Color32::DARK_GRAY);

    painter.text(
        response.rect.left_top() + (2.0, 5.0).into(),
        egui::Align2::LEFT_TOP,
        config.id(),
        egui::FontId::proportional(12.0),
        ecolor::Color32::WHITE,
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
                ecolor::Color32::WHITE,
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
            painter.rect_filled(response.rect, 0.0, ecolor::Color32::from_black_alpha(128));
        }
    }

    if let Some(right_top_text) = decoration.right_top_text {
        painter.text(
            response.rect.right_top() + (-2.0, 5.0).into(),
            egui::Align2::RIGHT_TOP,
            right_top_text,
            egui::FontId::monospace(9.0),
            ecolor::Color32::LIGHT_GRAY,
        );
    }
    if let Some(left_bottom_text) = decoration.left_bottom_text {
        painter.text(
            response.rect.left_bottom() + (2.0, -2.0).into(),
            egui::Align2::LEFT_BOTTOM,
            left_bottom_text,
            egui::FontId::monospace(9.0),
            ecolor::Color32::LIGHT_GRAY,
        );
    }

    if response.hovered() {
        painter.rect_stroke(
            response.rect,
            0.0,
            egui::Stroke::new(2.0, ecolor::Color32::WHITE),
            egui::StrokeKind::Middle,
        );
    }

    response
}
