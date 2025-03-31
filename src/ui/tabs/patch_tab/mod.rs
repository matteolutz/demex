use crate::ui::{components::tab_viewer::TabViewer, context::DemexUiContext};

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum PatchViewTab {
    PatchedFixtures,
    FixtureTypes,
    PatchNewFixture,
}

impl std::fmt::Display for PatchViewTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PatchNewFixture => write!(f, "Patch New Fixture"),
            Self::PatchedFixtures => write!(f, "Patched Fixtures"),
            Self::FixtureTypes => write!(f, "Fixture Types"),
        }
    }
}

impl PatchViewTab {
    pub fn ui(&self, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.label(self.to_string());
        });
    }
}

#[allow(dead_code)]
pub struct PatchViewComponent<'a> {
    context: &'a mut DemexUiContext,
    id_source: egui::Id,
}

impl<'a> PatchViewComponent<'a> {
    pub fn new(context: &'a mut DemexUiContext) -> Self {
        Self {
            context,
            id_source: egui::Id::new("DemexPatchViewComponent"),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            let response = TabViewer::new(
                "DemexPatchViewTabViewer",
                vec![
                    PatchViewTab::PatchedFixtures,
                    PatchViewTab::FixtureTypes,
                    PatchViewTab::PatchNewFixture,
                ],
                0,
            )
            .show(ui);

            ui.separator();

            response.selected_tab.ui(ui);
        });
    }
}
