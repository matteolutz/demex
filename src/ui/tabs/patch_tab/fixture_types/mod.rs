use itertools::Itertools;
use rfd::FileDialog;

use crate::{fixture::patch::FixtureFile, ui::context::DemexUiContext};

pub fn ui(ui: &mut egui::Ui, context: &mut DemexUiContext) {
    let mut fixture_handler = context.fixture_handler.write();

    ui.heading("Fixture Types");

    egui_extras::TableBuilder::new(ui)
        .auto_shrink(egui::Vec2b::new(false, true))
        .column(egui_extras::Column::auto())
        .columns(egui_extras::Column::remainder(), 2)
        .striped(true)
        .header(20.0, |mut ui| {
            ui.col(|ui| {
                ui.strong("Id");
            });

            ui.col(|ui| {
                ui.strong("Name");
            });

            ui.col(|ui| {
                ui.strong("Modes");
            });
        })
        .body(|mut ui| {
            for (fixture_type_id, fixture_type) in fixture_handler.patch().fixture_types() {
                ui.row(20.0, |mut ui| {
                    ui.col(|ui| {
                        ui.label(fixture_type_id);
                    });

                    ui.col(|ui| {
                        ui.label(&fixture_type.name);
                    });

                    ui.col(|ui| {
                        ui.label(
                            fixture_type
                                .modes
                                .iter()
                                .map(|(_, mode)| {
                                    format!("{} ({} channels)", mode.name, mode.channel_types.len())
                                })
                                .join(", "),
                        );
                    });
                });
            }
        });

    ui.separator();
    if ui.button("Import").clicked() {
        if let Some(file) = FileDialog::new()
            .add_filter("demex Fixture-File", &["json"])
            .pick_file()
        {
            let fixture_file: FixtureFile =
                serde_json::from_reader(std::fs::File::open(file).unwrap()).unwrap();

            fixture_handler
                .patch_mut()
                .fixture_types_mut()
                .insert(fixture_file.id, fixture_file.config);
        }
    }
}
