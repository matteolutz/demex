use egui_probe::Probe;

use crate::{
    fixture::{handler::FixtureHandler, selection::FixtureSelection},
    ui::utils::painter::painter_layout_centered,
};

use super::splitter::{Splitter, SplitterAxis};

const ELEMENT_SIZE: f32 = 50.0;

pub fn fixture_selection_editor(
    ui: &mut egui::Ui,
    fixture_selection: &mut FixtureSelection,
    fixture_handler: &FixtureHandler,
) {
    Splitter::new("FixtureSelectionTabSplitter", SplitterAxis::Horizontal).show(
        ui,
        |left_ui, right_ui| {
            egui::ScrollArea::both()
                .auto_shrink(emath::Vec2b::FALSE)
                .show(left_ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.style_mut().spacing.item_spacing = emath::vec2(10.0, 10.0);

                        for offset_idx in 0..fixture_selection.num_offsets() {
                            ui.vertical(|ui| {
                                ui.allocate_ui(emath::vec2(ELEMENT_SIZE, 10.0), |ui| {
                                    ui.centered_and_justified(|ui| {
                                        ui.label(offset_idx.to_string());
                                    });
                                });

                                for fixture_id in
                                    fixture_selection.fixtures_with_offset_idx(offset_idx)
                                {
                                    let fixture = fixture_handler.fixture_immut(fixture_id);

                                    // let _ = ui.button(format!("{}", fixture_id));
                                    let (response, painter) = ui.allocate_painter(
                                        emath::vec2(ELEMENT_SIZE, ELEMENT_SIZE),
                                        egui::Sense::hover(),
                                    );

                                    painter.rect_filled(response.rect, 2.0, ecolor::Color32::WHITE);
                                    painter_layout_centered(
                                        &painter,
                                        fixture
                                            .map(|f| f.name().to_owned())
                                            .unwrap_or_else(|| fixture_id.to_string()),
                                        egui::FontId::proportional(10.0),
                                        ecolor::Color32::BLACK,
                                        response.rect.shrink(5.0),
                                    );
                                }
                            });
                        }
                    });
                });

            right_ui.vertical(|ui| {
                ui.heading("Fixture Selection Attributes");

                ui.add_space(10.0);

                ui.label(format!(
                    "Fixtures selected: {}",
                    fixture_selection.fixtures().len()
                ));

                ui.label(format!("No. offsets: {}", fixture_selection.num_offsets()));

                ui.add_space(20.0);

                egui_extras::TableBuilder::new(ui)
                    .columns(egui_extras::Column::auto(), 4)
                    .cell_layout(egui::Layout::centered_and_justified(
                        egui::Direction::LeftToRight,
                    ))
                    .body(|mut ui| {
                        // Group
                        ui.row(20.0, |mut ui| {
                            ui.col(|ui| {
                                ui.label("Group");
                            });
                            ui.col(|ui| {
                                if ui.button("-").clicked() {
                                    *fixture_selection.group_mut() -= 1;
                                }
                            });

                            ui.col(|ui| {
                                egui_probe::Probe::new(fixture_selection.group_mut())
                                    .with_header("")
                                    .show(ui);
                            });
                            ui.col(|ui| {
                                if ui.button("+").clicked() {
                                    *fixture_selection.group_mut() += 1;
                                }
                            });
                        });

                        // Block
                        ui.row(20.0, |mut ui| {
                            ui.col(|ui| {
                                ui.label("Block");
                            });
                            ui.col(|ui| {
                                if ui.button("-").clicked() {
                                    *fixture_selection.block_mut() -= 1;
                                }
                            });

                            ui.col(|ui| {
                                egui_probe::Probe::new(fixture_selection.block_mut())
                                    .with_header("")
                                    .show(ui);
                            });
                            ui.col(|ui| {
                                if ui.button("+").clicked() {
                                    *fixture_selection.block_mut() += 1;
                                }
                            });
                        });

                        // Wings
                        ui.row(20.0, |mut ui| {
                            ui.col(|ui| {
                                ui.label("Wings");
                            });
                            ui.col(|ui| {
                                if ui.button("-").clicked() {
                                    *fixture_selection.wings_mut() -= 1;
                                }
                            });

                            ui.col(|ui| {
                                egui_probe::Probe::new(fixture_selection.wings_mut())
                                    .with_header("")
                                    .show(ui);
                            });
                            ui.col(|ui| {
                                if ui.button("+").clicked() {
                                    *fixture_selection.wings_mut() += 1;
                                }
                            });
                        });

                        // Reverse
                        ui.row(20.0, |mut ui| {
                            ui.col(|ui| {
                                ui.label("Reverse?");
                            });

                            ui.col(|_| {});

                            ui.col(|ui| {
                                Probe::new(fixture_selection.reverse_mut())
                                    .with_header("")
                                    .show(ui);
                            });

                            ui.col(|_| {});
                        });
                    });
            });
        },
    );
}
