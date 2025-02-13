use egui::RichText;
use itertools::Itertools;

use crate::{
    fixture::channel::{
        value::FixtureChannelValueTrait, FIXTURE_CHANNEL_COLOR_ID, FIXTURE_CHANNEL_INTENSITY_ID,
        FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
    },
    parser::nodes::fixture_selector::FixtureSelectorContext,
};

const SOURCE_INDEX_COLORS: [egui::Color32; 4] = [
    egui::Color32::YELLOW,
    egui::Color32::LIGHT_BLUE,
    egui::Color32::LIGHT_RED,
    egui::Color32::LIGHT_GREEN,
];

fn color_for_source_index(idx: usize) -> egui::Color32 {
    SOURCE_INDEX_COLORS[idx.min(SOURCE_INDEX_COLORS.len() - 1)]
}

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut super::DemexUiContext) {
    let fixture_handler = context.fixture_handler.read();
    let preset_handler = context.preset_handler.read();
    let updatable_handler = context.updatable_handler.read();

    egui::ScrollArea::horizontal().show(ui, |ui| {
        let selectd_fixtures = if let Some(fixture_select) = &context.global_fixture_select {
            fixture_select
                .get_fixtures(
                    &preset_handler,
                    FixtureSelectorContext::new(&context.global_fixture_select),
                )
                .unwrap_or(vec![])
        } else {
            vec![]
        };

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
                    ui.heading("Other Channels");
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
                                "{}.{}",
                                fixture.universe(),
                                fixture.start_address()
                            ));
                        });

                        row.col(|ui| {
                            ui.label(RichText::from(fixture.name()).strong().background_color(
                                if selectd_fixtures.contains(&fixture.id()) {
                                    egui::Color32::DARK_GREEN
                                } else {
                                    egui::Color32::TRANSPARENT
                                },
                            ));
                        });

                        // Value Sources
                        row.col(|ui| {
                            for (idx, source) in fixture.sources().iter().enumerate() {
                                ui.label(
                                    RichText::from(source.to_string())
                                        .color(color_for_source_index(idx)),
                                );
                            }
                        });

                        // Intens
                        row.col(|ui| {
                            if let Ok(intensity) =
                                fixture.intensity(&preset_handler, &updatable_handler)
                            {
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

                        // Color
                        row.col(|ui| {
                            if let Ok(color) = fixture.color(&preset_handler, &updatable_handler) {
                                ui.label(RichText::from(color.to_string(&preset_handler)).color(
                                    if color.is_home() {
                                        egui::Color32::GRAY
                                    } else {
                                        egui::Color32::YELLOW
                                    },
                                ));

                                let value = fixture
                                    .display_color(&preset_handler, &updatable_handler)
                                    .unwrap();

                                let color_value = egui::Color32::from_rgb(
                                    (value[0] * 255.0) as u8,
                                    (value[1] * 255.0) as u8,
                                    (value[2] * 255.0) as u8,
                                );

                                if !color.is_home() {
                                    ui.label(RichText::from("     ").background_color(color_value));
                                }
                            } else {
                                ui.label("-");
                            }
                        });

                        // Position
                        row.col(|ui| {
                            if let Ok(pos) =
                                fixture.position_pan_tilt(&preset_handler, &updatable_handler)
                            {
                                ui.label(RichText::from(pos.to_string(&preset_handler)).color(
                                    if pos.is_home() {
                                        egui::Color32::GRAY
                                    } else {
                                        egui::Color32::YELLOW
                                    },
                                ));
                            } else {
                                ui.label("-");
                            }
                        });

                        // Other channels
                        row.col(|ui| {
                            for channel_type in fixture.channel_types() {
                                if *channel_type == FIXTURE_CHANNEL_INTENSITY_ID
                                    || *channel_type == FIXTURE_CHANNEL_COLOR_ID
                                    || *channel_type == FIXTURE_CHANNEL_POSITION_PAN_TILT_ID
                                {
                                    continue;
                                }

                                let channel_value = fixture.channel_value(
                                    *channel_type,
                                    &preset_handler,
                                    &updatable_handler,
                                );
                                if channel_value.is_err() {
                                    continue;
                                }

                                if channel_value.as_ref().unwrap().is_home() {
                                    continue;
                                }

                                ui.label(
                                    RichText::from(format!(
                                        "{}({})",
                                        fixture.channel_name(*channel_type).unwrap(),
                                        channel_value.as_ref().unwrap().to_string(&preset_handler)
                                    ))
                                    .color(egui::Color32::YELLOW),
                                );
                            }
                        });
                    });
                }
            });

        /*ui.with_layout(
            eframe::egui::Layout::top_down(eframe::egui::Align::LEFT),
            |ui| {
                eframe::egui::ScrollArea::vertical()
                    .max_height(ui.available_height())
                    .max_width(ui.available_width())
                    .show(ui, |ui| {
                        ui.heading("Fixtures");

                        let selectd_fixtures =
                            if let Some(fixture_select) = &context.global_fixture_select {
                                fixture_select
                                    .get_fixtures(
                                        &context.preset_handler,
                                        FixtureSelectorContext::new(&context.global_fixture_select),
                                    )
                                    .unwrap_or(vec![])
                            } else {
                                vec![]
                            };

                        for fixture_chunk in context
                            .fixture_handler
                            .fixtures()
                            .chunks((window_width / fixture_card_size.x) as usize - 1)
                        {
                            ui.horizontal(|ui| {
                                for f in fixture_chunk {
                                    let fixture_intenstiy = f
                                        .intensity()
                                        .expect("")
                                        .as_single(&context.preset_handler, f.id())
                                        .expect("todo: error handling for intensity");

                                    let (rect, response) = ui.allocate_exact_size(
                                        fixture_card_size,
                                        eframe::egui::Sense::click(),
                                    );

                                    if response.clicked() {
                                        // TODO: make this work
                                        println!("fixture clicked: {}", f.id());
                                    }

                                    ui.painter().rect_stroke(
                                        rect,
                                        10.0,
                                        eframe::egui::Stroke::new(
                                            2.0,
                                            if selectd_fixtures.contains(&f.id()) {
                                                eframe::egui::Color32::from_rgb(0, 255, 0)
                                            } else {
                                                eframe::egui::Color32::from_rgb(
                                                    255,
                                                    255,
                                                    255 - (fixture_intenstiy * 255.0) as u8,
                                                )
                                            },
                                        ),
                                    );

                                    ui.put(rect, |ui: &mut eframe::egui::Ui| {
                                        ui.colored_label(
                                            eframe::egui::Color32::from_rgb(
                                                255,
                                                255,
                                                255 - (fixture_intenstiy * 255.0) as u8,
                                            ),
                                            RichText::from(f.to_string(&context.preset_handler))
                                                .strong(),
                                        )
                                    });
                                }
                            });
                        }

                        ui.add_space(ui.available_height());
                    });
            },
        );*/
    });
}
