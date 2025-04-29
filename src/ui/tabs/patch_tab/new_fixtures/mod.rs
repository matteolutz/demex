use crate::{
    fixture::patch::FixtureTypeAndMode,
    ui::{
        context::DemexUiContext,
        dlog::dialog::DemexGlobalDialogEntry,
        patch::{template::render_new_fixture_patch_name, PatchUiNewFixture},
    },
};

#[derive(Default, Clone)]
struct State {
    pub fixture_type: Option<String>,
    pub fixture_mode: Option<u32>,

    pub universe: u16,
    pub start_address: u16,
    pub address_padding: u16,

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
                        ui.label("Adress Padding");
                    });

                    ui.col(|ui| {
                        egui_probe::Probe::new(&mut state.address_padding)
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
                todo!();
                /*
                let fixture_type_id = state.fixture_type.clone().unwrap();
                let fixture_type = patch.fixture_types().get(&fixture_type_id).unwrap();

                let fixture_mode_id = state.fixture_mode.unwrap();
                let fixture_mode = fixture_type.modes.get(&fixture_mode_id).unwrap();

                let dmx_footprint = fixture_mode.channel_types.len() as u16;

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
                    for idx in 0..state.num_fixutres {
                        let fixture_id = state.start_id + idx as u32;
                        let fixture_address = state.start_address
                            + (idx * dmx_footprint)
                            + idx * state.address_padding;

                        let fixture_name = render_new_fixture_patch_name(
                            PatchUiNewFixture {
                                id: fixture_id,
                                universe: state.universe,
                                start_address: state.start_address,
                                type_and_mode: FixtureTypeAndMode {
                                    name: fixture_type_id.clone(),
                                    mode: fixture_mode_id,
                                },
                            },
                            &state.name_format,
                        )
                        .unwrap();


                        patch.fixtures_mut().push(SerializableFixturePatch {
                            id: fixture_id,
                            name: fixture_name,
                            fixture_type: fixture_type_id.clone(),
                            fixture_mode: fixture_mode_id,

                            channel_modifiers: Vec::new(),

                            universe: state.universe,
                            start_address: fixture_address,
                        });
                    }

                    fixture_handler.reload_patch();

                    drop(fixture_handler);
                    self.context.add_dialog_entry(DemexGlobalDialogEntry::info(
                        format!("Patched {} fixtures", state.num_fixutres).as_str(),
                    ));


                }

                */
            }
        }

        ui.data_mut(|data| data.insert_temp(self.id_source, state));
    }
}
