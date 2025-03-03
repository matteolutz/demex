use std::collections::VecDeque;

use strum::{EnumIter, IntoEnumIterator};

use crate::ui::utils::painter::painter_layout_centered;

const CENTER_OFFSET: (f32, f32) = (60.0, 60.0);

const ACTION_SIZE: f32 = 55.0;
const ACTION_ROUNDING: f32 = 5.0;

pub struct QuickMenuAction<T: Copy> {
    name: String,
    id: T,
}

impl<T: Copy> From<(T, String)> for QuickMenuAction<T> {
    fn from((id, name): (T, String)) -> Self {
        Self { id, name }
    }
}

impl<T: Copy> From<(T, &str)> for QuickMenuAction<T> {
    fn from((id, name): (T, &str)) -> Self {
        Self {
            id,
            name: name.to_owned(),
        }
    }
}

#[derive(EnumIter, Copy, Clone)]
pub enum QuickMenuActionPosition {
    TopLeft,
    TopCenter,
    TopRight,

    LeftCenter,
    RightCenter,

    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl QuickMenuActionPosition {
    pub fn idx(&self) -> usize {
        match self {
            Self::TopLeft => 0,
            Self::TopRight => 1,
            Self::BottomLeft => 2,
            Self::BottomRight => 3,
            Self::TopCenter => 4,
            Self::BottomCenter => 5,
            Self::LeftCenter => 6,
            Self::RightCenter => 7,
        }
    }

    pub fn offset(&self) -> egui::Vec2 {
        match self {
            Self::TopLeft => egui::vec2(-1.0, -1.0),
            Self::TopRight => egui::vec2(1.0, -1.0),
            Self::BottomLeft => egui::vec2(-1.0, 1.0),
            Self::BottomRight => egui::vec2(1.0, 1.0),
            Self::TopCenter => egui::vec2(0.0, -1.0),
            Self::BottomCenter => egui::vec2(0.0, 1.0),
            Self::LeftCenter => egui::vec2(-1.0, 0.0),
            Self::RightCenter => egui::vec2(1.0, 0.0),
        }
    }

    pub fn rounding(&self) -> egui::Rounding {
        match self {
            Self::TopLeft => egui::Rounding {
                nw: ACTION_ROUNDING,
                ne: 0.0,
                se: 0.0,
                sw: 0.0,
            },
            Self::TopRight => egui::Rounding {
                nw: 0.0,
                ne: ACTION_ROUNDING,
                se: 0.0,
                sw: 0.0,
            },
            Self::BottomLeft => egui::Rounding {
                nw: 0.0,
                ne: 0.0,
                se: 0.0,
                sw: ACTION_ROUNDING,
            },
            Self::BottomRight => egui::Rounding {
                nw: 0.0,
                ne: 0.0,
                se: ACTION_ROUNDING,
                sw: 0.0,
            },

            Self::TopCenter | Self::BottomCenter | Self::LeftCenter | Self::RightCenter => {
                egui::Rounding::ZERO
            }
        }
    }
}

pub struct QuickMenuActions<T: Copy> {
    actions: [Option<QuickMenuAction<T>>; 8],
}

impl<T: Copy> Default for QuickMenuActions<T> {
    fn default() -> Self {
        Self {
            actions: Default::default(),
        }
    }
}

impl<T: Copy> QuickMenuActions<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn actions(&self) -> &[Option<QuickMenuAction<T>>; 8] {
        &self.actions
    }

    pub fn action(&self, position: QuickMenuActionPosition) -> Option<&QuickMenuAction<T>> {
        self.actions[position.idx()].as_ref()
    }

    pub fn set_action(&mut self, position: QuickMenuActionPosition, action: QuickMenuAction<T>) {
        self.actions[position.idx()] = Some(action);
    }

    pub fn top_left(mut self, action: impl Into<QuickMenuAction<T>>) -> Self {
        self.set_action(QuickMenuActionPosition::TopLeft, action.into());
        self
    }

    pub fn top_right(mut self, action: impl Into<QuickMenuAction<T>>) -> Self {
        self.set_action(QuickMenuActionPosition::TopRight, action.into());
        self
    }

    pub fn bottom_left(mut self, action: impl Into<QuickMenuAction<T>>) -> Self {
        self.set_action(QuickMenuActionPosition::BottomLeft, action.into());
        self
    }

    pub fn bottom_right(mut self, action: impl Into<QuickMenuAction<T>>) -> Self {
        self.set_action(QuickMenuActionPosition::BottomRight, action.into());
        self
    }

    pub fn top_center(mut self, action: impl Into<QuickMenuAction<T>>) -> Self {
        self.set_action(QuickMenuActionPosition::TopCenter, action.into());
        self
    }

    pub fn bottom_center(mut self, action: impl Into<QuickMenuAction<T>>) -> Self {
        self.set_action(QuickMenuActionPosition::BottomCenter, action.into());
        self
    }

    pub fn left_center(mut self, action: impl Into<QuickMenuAction<T>>) -> Self {
        self.set_action(QuickMenuActionPosition::LeftCenter, action.into());
        self
    }

    pub fn right_center(mut self, action: impl Into<QuickMenuAction<T>>) -> Self {
        self.set_action(QuickMenuActionPosition::RightCenter, action.into());
        self
    }

    pub fn with_vec(mut self, mut actions: VecDeque<impl Into<QuickMenuAction<T>>>) -> Self {
        for position in QuickMenuActionPosition::iter() {
            if self.actions[position.idx()].is_some() {
                continue;
            }

            if actions.is_empty() {
                break;
            }

            self.set_action(position, actions.pop_front().unwrap().into());
        }

        self
    }
}

pub type QuickMenuResponse<T> = Option<T>;

pub struct QuickMenu<'a, T: Copy> {
    pivot: egui::Pos2,
    actions: &'a QuickMenuActions<T>,
}

impl<'a, T: Copy> QuickMenu<'a, T> {
    pub fn new(pivot: egui::Pos2, actions: &'a QuickMenuActions<T>) -> Self {
        Self { pivot, actions }
    }

    fn action_rect(&self, position: QuickMenuActionPosition) -> egui::Rect {
        egui::Rect::from_center_size(
            self.pivot + (position.offset() * egui::Vec2::from(CENTER_OFFSET)),
            egui::vec2(ACTION_SIZE, ACTION_SIZE),
        )
    }

    #[allow(dead_code)]
    fn bounding_rect(&self, padding: f32) -> egui::Rect {
        let half_size = ACTION_SIZE / 2.0;
        let padding = egui::vec2(padding, padding);

        let min = self.pivot
            - egui::vec2(CENTER_OFFSET.0 + half_size, CENTER_OFFSET.1 + half_size)
            - padding;

        let max = self.pivot
            + egui::vec2(CENTER_OFFSET.0 + half_size, CENTER_OFFSET.1 + half_size)
            + padding;

        egui::Rect::from_min_max(min, max)
    }

    pub fn interact(&self, pos: egui::Pos2) -> QuickMenuResponse<T> {
        QuickMenuActionPosition::iter()
            .find(|&position| self.action_rect(position).contains(pos))
            .and_then(|position| self.actions.action(position).map(|action| action.id))
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let overlay_painter = ui.ctx().layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            "demex::QuickMenu".into(),
        ));

        /*overlay_painter.rect_filled(
            self.bounding_rect(10.0),
            5.0,
            egui::Color32::BLACK.gamma_multiply(0.5),
        );*/

        for position in QuickMenuActionPosition::iter() {
            if let Some(action) = self.actions.action(position) {
                let rect = self.action_rect(position);

                let is_hovered = ui
                    .input(|reader| reader.pointer.hover_pos())
                    .is_some_and(|pos| rect.contains(pos));

                overlay_painter.rect_filled(
                    rect,
                    position.rounding(),
                    egui::Color32::from_gray(if is_hovered { 150 } else { 125 }),
                );

                painter_layout_centered(
                    &overlay_painter,
                    action.name.clone(),
                    egui::FontId::proportional(10.0),
                    egui::Color32::WHITE,
                    rect,
                );
            }
        }
    }
}
