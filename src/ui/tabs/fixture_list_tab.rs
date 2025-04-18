use egui::RichText;
use itertools::Itertools;

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut super::DemexUiContext) {
    let fixture_handler = context.fixture_handler.read();
    let preset_handler = context.preset_handler.read();
    let updatable_handler = context.updatable_handler.read();
    let timing_handler = context.timing_handler.read();

    egui::ScrollArea::horizontal().show(ui, |ui| {
        let selected_fixtures = context
            .global_fixture_select
            .as_ref()
            .map(|selection| selection.fixtures())
            .unwrap_or_default();

        egui_extras::TableBuilder::new(ui)
            .columns(egui_extras::Column::auto(), 7)
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
                    ui.heading("Color");
                });

                header.col(|ui| {
                    ui.heading("Pos");
                });

                header.col(|ui| {
                    ui.heading("All Channels");
                });
            })
            .body(|mut body| {
                for fixture in fixture_handler
                    .fixtures()
                    .iter()
                    .sorted_by(|a, b| a.id().cmp(&b.id()))
                {
                    body.row(50.0, |mut row| {
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
                                    RichText::from(source.to_short_string())
                                        .color(source.get_color()),
                                );
                            }
                        });

                        // Intens
                        row.col(|ui| {
                            if let Ok(intensity) = fixture.get_attribute_value(
                                &fixture_handler,
                                "Dimmer",
                                &preset_handler,
                                &updatable_handler,
                                &timing_handler,
                            ) {
                                ui.label(
                                    RichText::from(intensity.to_string(&preset_handler)).color(
                                        if intensity.is_home() {
                                            egui::Color32::GRAY
                                        } else {
                                            egui::Color32::YELLOW
                                        },
                                    ),
                                );
                            } else {
                                ui.label("-");
                            }
                        });

                        /*
                        // Color
                        row.col(|ui| {
                            if let Ok(color_states) = fixture.feature_group_display_state(
                                DefaultFeatureGroup::Color.id(),
                                &preset_handler,
                                &updatable_handler,
                                &timing_handler,
                            ) {
                                for color_state in color_states {
                                    ui.label(
                                        RichText::from(color_state.to_string(&preset_handler))
                                            .color(if color_state.is_home() {
                                                egui::Color32::GRAY
                                            } else {
                                                egui::Color32::YELLOW
                                            }),
                                    );
                                }
                            } else {
                                ui.label("-");
                            }

                            if let Ok(color) = fixture.display_color(
                                &preset_handler,
                                &updatable_handler,
                                &timing_handler,
                            ) {
                                let color_value = egui::Color32::from_rgb(
                                    (color[0] * 255.0) as u8,
                                    (color[1] * 255.0) as u8,
                                    (color[2] * 255.0) as u8,
                                );

                                ui.label(RichText::from("     ").background_color(color_value));
                            }
                        });

                        // Position
                        row.col(|ui| {
                            if let Ok(pos_states) = fixture.feature_group_display_state(
                                DefaultFeatureGroup::Position.id(),
                                &preset_handler,
                                &updatable_handler,
                                &timing_handler,
                            ) {
                                for pos_state in pos_states {
                                    ui.label(
                                        RichText::from(pos_state.to_string(&preset_handler)).color(
                                            if pos_state.is_home() {
                                                egui::Color32::GRAY
                                            } else {
                                                egui::Color32::YELLOW
                                            },
                                        ),
                                    );
                                }
                            } else {
                                ui.label("-");
                            }
                        });


                        // All channels
                        row.col(|ui| {
                            for channel_type in fixture.channel_types().iter().sorted() {
                                let channel_value = fixture.channel_value(
                                    **channel_type,
                                    &updatable_handler,
                                    &preset_handler,
                                    &timing_handler,
                                );
                                if channel_value.is_err() {
                                    continue;
                                }

                                if channel_value.as_ref().unwrap().is_home() {
                                    continue;
                                }

                                ui.label(
                                    RichText::from(channel_type.short_name().to_string())
                                        .color(egui::Color32::YELLOW),
                                );
                            }
                        });

                        */
                    });
                }
            });
    });
}
