use egui::RichText;

use crate::{
    fixture::channel::value::FixtureChannelValueTrait,
    parser::nodes::fixture_selector::FixtureSelectorContext,
};

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut super::DemexUiContext) {
    let fixture_card_size = eframe::egui::vec2(75.0 * 1.5, 100.0 * 1.5);
    let window_width = ui.available_width();

    ui.with_layout(
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
    );
}
