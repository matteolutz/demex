use crate::ui::{components::tab_viewer::TabViewer, context::DemexUiContext};

pub mod fixture_types;
pub mod new_fixtures;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum PatchViewTab {
    PatchedFixtures,
    FixtureTypes,
    PatchNewFixtures,
}

impl std::fmt::Display for PatchViewTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PatchNewFixtures => write!(f, "Patch New Fixtures"),
            Self::PatchedFixtures => write!(f, "Patched Fixtures"),
            Self::FixtureTypes => write!(f, "Fixture Types"),
        }
    }
}

impl PatchViewTab {
    pub fn ui(&self, ui: &mut egui::Ui, context: &mut DemexUiContext) {
        match self {
            Self::PatchNewFixtures => {
                new_fixtures::PatchNewFixturesComponent::new(context).show(ui)
            }
            Self::FixtureTypes => fixture_types::ui(ui, context),
            _ => {
                ui.centered_and_justified(|ui| {
                    ui.label(self.to_string());
                });
            }
        }
    }
}

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
                self.id_source,
                vec![
                    PatchViewTab::PatchedFixtures,
                    PatchViewTab::FixtureTypes,
                    PatchViewTab::PatchNewFixtures,
                ],
                0,
            )
            .show(ui);

            ui.separator();

            response.selected_tab.ui(ui, self.context);
        });
    }
}
