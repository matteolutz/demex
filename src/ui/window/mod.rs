use std::sync::Arc;

use edit::DemexEditWindow;
use parking_lot::RwLock;

use crate::fixture::{
    handler::FixtureHandler, patch::Patch, presets::PresetHandler, updatables::UpdatableHandler,
};

use super::dlog::dialog::DemexGlobalDialogEntry;

pub mod edit;

#[derive(Default)]
pub struct DemexWindowHandler {
    windows: Vec<DemexWindow>,
}

impl DemexWindowHandler {
    pub fn add_window(&mut self, window: DemexWindow) {
        if self.windows.contains(&window) {
            return;
        }

        self.windows.push(window);
    }

    pub fn add_dialog_window(&mut self, dialog_entry: DemexGlobalDialogEntry) {
        let dialog_window = self.windows.iter_mut().find(|w| w.is_dialog());

        if let Some(dialog_window) = dialog_window {
            dialog_window.add_dialog_entry(dialog_entry.clone());
        } else {
            self.windows
                .push(DemexWindow::Dialog(vec![dialog_entry.clone()]));
        }
    }

    pub fn clear(&mut self) {
        self.windows.retain(|w| !w.is_dialog());
    }

    pub fn is_empty(&mut self) -> bool {
        self.windows.is_empty()
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        fixture_handler: &mut Arc<RwLock<FixtureHandler>>,
        preset_handler: &mut Arc<RwLock<PresetHandler>>,
        updatable_handler: &mut Arc<RwLock<UpdatableHandler>>,
        patch: &mut Arc<RwLock<Patch>>,
    ) {
        for i in 0..self.windows.len() {
            if self.windows[i].ui(
                ctx,
                fixture_handler,
                preset_handler,
                updatable_handler,
                patch,
            ) {
                self.windows.remove(i);
            }
        }
    }
}

#[derive(Debug, Eq)]
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
            (Self::AboutDemex, Self::AboutDemex) => true,
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

    pub fn should_fullscreen(&self) -> bool {
        match self {
            Self::Edit(edit_window) => edit_window.should_fullscreen(),
            _ => false,
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
        patch: &mut Arc<RwLock<Patch>>,
    ) -> bool {
        let mut window = egui::Window::new(self.title())
            .interactable(true)
            .collapsible(false)
            .scroll(true)
            .order(self.order());

        window = if self.should_fullscreen() {
            window.fixed_rect(ctx.screen_rect())
        } else {
            let screen_rect = ctx.screen_rect();
            let shrunk_rect = screen_rect.shrink2(egui::vec2(
                screen_rect.width() * 0.25,
                screen_rect.height() * 0.25,
            ));

            window.fixed_rect(shrunk_rect)
        };

        window
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
                        let mut patch = patch.write();

                        if let Err(err) = edit_window.window_ui(
                            ui,
                            &mut fixture_handler,
                            &mut preset_handler,
                            &mut updatable_handler,
                            &mut patch,
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

                ui.add_space(20.0);
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
