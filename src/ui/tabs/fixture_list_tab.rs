use egui::RichText;
use itertools::Itertools;

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut super::DemexUiContext) {
    let fixture_handler = context.fixture_handler.read();
    let preset_handler = context.preset_handler.read();
    let patch = context.patch.read();

    egui::ScrollArea::horizontal().show(ui, |ui| {
        let selected_fixtures = context
            .global_fixture_select
            .as_ref()
            .map(|selection| selection.fixtures())
            .unwrap_or_default();

        egui_extras::TableBuilder::new(ui)
            .columns(egui_extras::Column::auto(), 5)
            .column(egui_extras::Column::remainder())
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .striped(true)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading("Id");
                });

                header.col(|ui| {
                    ui.heading("Addr");
                });

                header.col(|ui| {
                    ui.heading("Name");
                });

                header.col(|ui| {
                    ui.heading("Src");
                });

                header.col(|ui| {
                    ui.heading("Int");
                });

                header.col(|ui| {
                    ui.heading("All Channels");
                });
            })
            .body(|body| {
                let mut fixture_iter = fixture_handler
                    .fixtures()
                    .iter()
                    .sorted_by(|a, b| a.id().cmp(&b.id()));

                body.rows(50.0, fixture_handler.fixtures().len(), |mut row| {
                    let fixture = fixture_iter.next().unwrap();

                    row.col(|ui| {
                        ui.label(fixture.id().to_string());
                    });

                    row.col(|ui| {
                        ui.label(format!(
                            "U{}.{}",
                            fixture.universe(),
                            fixture.start_address()
                        ));
                    });

                    row.col(|ui| {
                        ui.label(RichText::from(fixture.name()).strong().background_color(
                            if selected_fixtures.contains(&fixture.id()) {
                                egui::Color32::DARK_GREEN
                            } else {
                                egui::Color32::TRANSPARENT
                            },
                        ));
                    });

                    // Value Sources
                    row.col(|ui| {
                        for source in fixture.sources() {
                            ui.label(
                                RichText::from(source.to_short_string()).color(source.get_color()),
                            );
                        }
                    });

                    // Intens
                    row.col(|ui| {
                        if let Ok(intensity) =
                            fixture.get_attribute_value(patch.fixture_types(), "Dimmer")
                        {
                            ui.label(RichText::from(intensity.to_string(&preset_handler)).color(
                                if intensity.is_home() {
                                    egui::Color32::GRAY
                                } else {
                                    egui::Color32::YELLOW
                                },
                            ));
                        } else {
                            ui.label("-");
                        }
                    });

                    // All channels
                    row.col(|ui| {
                        for (dmx_channel, _) in fixture.channels(patch.fixture_types()).unwrap() {
                            let channel_value = fixture
                                .get_value(patch.fixture_types(), dmx_channel.name().as_ref())
                                .unwrap();

                            if channel_value.is_home() {
                                continue;
                            }

                            ui.label(
                                RichText::from(dmx_channel.name().as_ref())
                                    .color(egui::Color32::YELLOW),
                            );
                        }
                    });
                });
            });
    });
}
