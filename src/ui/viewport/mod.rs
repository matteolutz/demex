use serde::{Deserialize, Serialize};

use crate::ui::{
    constants::MAX_VIEWPORTS,
    context::DemexUiContext,
    tabs::{DemexTab, DemexTabs},
    viewport::{position::DemexViewportPositonState, viewport_type::ViewportType},
};

pub mod position;
pub mod viewport_type;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemexViewport {
    viewport_type: ViewportType,
    tabs: Option<DemexTabs>,
    pos_state: DemexViewportPositonState,
}

impl DemexViewport {
    pub fn main() -> Self {
        Self {
            viewport_type: ViewportType::Main,
            tabs: Some(DemexTabs::main()),
            pos_state: DemexViewportPositonState::default(),
        }
    }

    pub fn new(id: impl Into<egui::Id>) -> Self {
        Self {
            viewport_type: ViewportType::Other {
                id: id.into(),
                is_active: true,
            },
            tabs: None,
            pos_state: DemexViewportPositonState::default(),
        }
    }

    pub fn default_viewports() -> [Self; MAX_VIEWPORTS] {
        core::array::from_fn(|i| {
            if i == 0 {
                DemexViewport::main()
            } else {
                DemexViewport::new(format!("DemexViewport{}", i))
            }
        })
    }

    pub fn should_render(&self) -> bool {
        self.viewport_type.should_render()
    }

    pub fn pos_state(&self) -> &DemexViewportPositonState {
        &self.pos_state
    }

    pub fn pos_state_mut(&mut self) -> &mut DemexViewportPositonState {
        &mut self.pos_state
    }

    pub fn focus(&mut self, tab: DemexTab) -> bool {
        self.tabs.as_mut().is_some_and(|tabs| tabs.focus(tab))
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, context: &mut DemexUiContext) {
        if self.tabs.is_none() {
            self.tabs = Some(DemexTabs::default());
        }

        self.tabs.as_mut().unwrap().ui(
            ui,
            context,
            self.viewport_type.id().with("TabViewer"),
            self.viewport_type.is_main(),
        );
    }
}
