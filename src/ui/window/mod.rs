use std::sync::Arc;

use edit::DemexEditWindow;
use parking_lot::RwLock;

use crate::fixture::{
    handler::FixtureHandler, presets::PresetHandler, updatables::UpdatableHandler,
};

use super::log::dialog::DemexGlobalDialogEntry;

pub mod edit;

#[derive(Debug)]
pub enum DemexWindow {
    Dialog(Vec<DemexGlobalDialogEntry>),
    Edit(DemexEditWindow),
    AboutDemex,
}

impl PartialEq for DemexWindow {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Dialog(_), Self::Dialog(_)) => false,
            (Self::Edit(edit_window), Self::Edit(other_edit_window)) => {
                edit_window == other_edit_window
            }
            _ => false,
        }
    }
}

impl DemexWindow {
    pub fn is_dialog(&self) -> bool {
        matches!(self, Self::Dialog(_))
    }

    pub fn add_dialog_entry(&mut self, entry: DemexGlobalDialogEntry) {
        if let Self::Dialog(entries) = self {
            entries.push(entry);
        }
    }
}

impl DemexWindow {
    pub fn title(&self) -> String {
        match self {
            Self::Dialog(_) => "demex dialog".to_owned(),
            Self::Edit(edit_window) => edit_window.title(),
            Self::AboutDemex => "About demex".to_owned(),
        }
    }

    pub fn order(&self) -> egui::Order {
        match self {
            Self::Dialog(_) => egui::Order::Foreground,
            Self::Edit(_) => egui::Order::Middle,
            Self::AboutDemex => egui::Order::Middle,
        }
    }

    pub fn ui(
        &self,
        ctx: &egui::Context,
        fixture_handler: &mut Arc<RwLock<FixtureHandler>>,
        preset_handler: &mut Arc<RwLock<PresetHandler>>,
        updatable_handler: &mut Arc<RwLock<UpdatableHandler>>,
    ) -> bool {
        egui::Window::new(self.title())
            .resizable(false)
            .movable(true)
            .interactable(true)
            .collapsible(false)
            .order(self.order())
            .show(ctx, |ui| {
                match self {
                    Self::Dialog(dialog_entries) => {
                        ui.vertical(|ui| {
                            for (idx, entry) in dialog_entries.iter().enumerate() {
                                entry.window_ui(ui);

                                if idx < dialog_entries.len() - 1 {
                                    ui.separator();
                                }
                            }
                        });
                    }
                    Self::Edit(edit_window) => {
                        let mut fixture_handler = fixture_handler.write();
                        let mut preset_handler = preset_handler.write();
                        let mut updatable_handler = updatable_handler.write();

                        if let Err(err) = edit_window.window_ui(
                            ui,
                            &mut fixture_handler,
                            &mut preset_handler,
                            &mut updatable_handler,
                        ) {
                            ui.vertical(|ui| {
                                ui.colored_label(egui::Color32::LIGHT_RED, "Something went wrong.");
                                ui.label(err.to_string());
                            });
                        }
                    }
                    Self::AboutDemex => {
                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                            ui.image(egui::include_image!(
                                "../../../assets/LogoV1-Wide-Title.png"
                            ));
                        });
                    }
                };

                if ui.button("Close").clicked() {
                    return true;
                }

                false
            })
            .map(|inner| inner.inner)
            .unwrap_or(None)
            .unwrap_or(false)
    }
}
