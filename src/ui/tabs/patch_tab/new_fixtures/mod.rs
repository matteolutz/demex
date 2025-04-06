use crate::ui::{context::DemexUiContext, dlog::dialog::DemexGlobalDialogEntry};

#[derive(Default, Clone)]
struct State {
    pub fixture_type: Option<String>,
    pub fixture_mode: Option<u32>,

    pub universe: u16,
    pub start_address: u16,

    pub start_id: u32,
    pub num_fixutres: u16,

    pub name_format: String,
}

pub struct PatchNewFixturesComponent<'a> {
    context: &'a mut DemexUiContext,
    id_source: egui::Id,
}

impl<'a> PatchNewFixturesComponent<'a> {
    pub fn new(context: &'a mut DemexUiContext) -> Self {
        Self {
            context,
            id_source: egui::Id::new("DemexPatchNewFixturesComponent"),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let mut state = ui
            .data(|data| data.get_temp::<State>(self.id_source))
            .unwrap_or_default();

        let mut fixture_handler = self.context.fixture_handler.write();
        let patch = fixture_handler.patch_mut();

        egui_extras::TableBuilder::new(ui)
            .column(egui_extras::Column::auto())
            .column(egui_extras::Column::remainder())
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .striped(true)
            .auto_shrink(egui::Vec2b::new(false, true))
            .body(|mut ui| {
                ui.row(70.0, |mut ui| {
                    ui.col(|ui| {
                        ui.label("Fixture type and mode");
                    });

                    ui.col(|ui| {
                        let fixture_types = patch.fixture_types();

                        egui::ComboBox::new(self.id_source.with("FixtureTypeSelection"), "")
                            .selected_text(
                                state
                                    .fixture_type
                                    .as_ref()
                                    .map(|fixture_type| {
                                        fixture_types.get(fixture_type).unwrap().name.clone()
                                    })
                                    .unwrap_or("None".to_owned()),
                            )
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut state.fixture_type, None, "None");

                                for (fixture_type_id, fixture_type) in fixture_types {
                                    ui.selectable_value(
                                        &mut state.fixture_type,
                                        Some(fixture_type_id.clone()),
                                        &fixture_type.name,
                                    );
                                }
                            });

                        if let Some(fixture_type) = &state.fixture_type {
                            let fixture_modes = &fixture_types.get(fixture_type).unwrap().modes;

                            let fixture_mode =
                                state.fixture_mode.and_then(|mode| fixture_modes.get(&mode));

                            egui::ComboBox::new(self.id_source.with("FixtureModeSelection"), "")
                                .selected_text(
                                    fixture_mode.map_or("None".to_owned(), |m| m.name.clone()),
                                )
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut state.fixture_mode, None, "None");

                                    for (mode_id, mode) in fixture_modes {
                                        ui.selectable_value(
                                            &mut state.fixture_mode,
                                            Some(*mode_id),
                                            format!(
                                                "{} ({} Channels)",
                                                mode.name,
                                                mode.channel_types.len()
                                            ),
                                        );
                                    }
                                });
                        }
                    });
                });

                ui.row(70.0, |mut ui| {
                    ui.col(|ui| {
                        ui.label("Universe");
                    });

                    ui.col(|ui| {
                        egui_probe::Probe::new(&mut state.universe)
                            .with_header("")
                            .show(ui);
                    });
                });

                ui.row(70.0, |mut ui| {
                    ui.col(|ui| {
                        ui.label("Start Address");
                    });

                    ui.col(|ui| {
                        egui_probe::Probe::new(&mut state.start_address)
                            .with_header("")
                            .show(ui);
                    });
                });

                ui.row(70.0, |mut ui| {
                    ui.col(|ui| {
                        ui.label("Start Id");
                    });

                    ui.col(|ui| {
                        egui_probe::Probe::new(&mut state.start_id)
                            .with_header("")
                            .show(ui);
                    });
                });

                ui.row(70.0, |mut ui| {
                    ui.col(|ui| {
                        ui.label("No. Fixtures");
                    });

                    ui.col(|ui| {
                        egui_probe::Probe::new(&mut state.num_fixutres)
                            .with_header("")
                            .show(ui);
                    });
                });

                ui.row(70.0, |mut ui| {
                    ui.col(|ui| {
                        ui.label("Name format");
                    });

                    ui.col(|ui| {
                        egui_probe::Probe::new(&mut state.name_format)
                            .with_header("")
                            .show(ui);
                    });
                });
            });

        if ui.button("Patch").clicked() {
            if state.fixture_type.is_none()
                || state.fixture_mode.is_none()
                || state.name_format.is_empty()
                || state.num_fixutres == 0
            {
                drop(fixture_handler);

                self.context
                    .add_dialog_entry(DemexGlobalDialogEntry::error_str(
                        "Invalid patch configuration!".to_owned(),
                    ));
            } else {
                let fixture_type = patch
                    .fixture_types()
                    .get(state.fixture_type.as_ref().unwrap())
                    .unwrap();
                let fixture_mode = fixture_type
                    .modes
                    .get(state.fixture_mode.as_ref().unwrap())
                    .unwrap();

                let address_range = state.start_address
                    ..(state.start_address
                        + (state.num_fixutres * fixture_mode.channel_types.len() as u16));

                if !patch.is_address_range_unpatched(address_range.clone(), state.universe) {
                    drop(fixture_handler);
                    self.context
                        .add_dialog_entry(DemexGlobalDialogEntry::error_str(format!(
                            "Patch range ({:?}) in universe {} is not unused.",
                            address_range, state.universe
                        )));
                } else {
                    // Insert into patch
                }
            }
        }

        ui.data_mut(|data| data.insert_temp(self.id_source, state));
    }
}
