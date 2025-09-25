use serde::{Deserialize, Serialize};

use crate::ui::constants::MAIN_VIEWPORT_ID;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ViewportType {
    Main,
    Other { id: egui::Id, is_active: bool },
}

impl ViewportType {
    pub fn id(&self) -> egui::Id {
        match self {
            ViewportType::Main => egui::Id::new(MAIN_VIEWPORT_ID),
            ViewportType::Other { id, .. } => *id,
        }
    }

    pub fn is_main(&self) -> bool {
        matches!(self, ViewportType::Main)
    }

    pub fn should_render(&self) -> bool {
        match self {
            ViewportType::Main => true,
            ViewportType::Other { is_active, .. } => *is_active,
        }
    }
}
