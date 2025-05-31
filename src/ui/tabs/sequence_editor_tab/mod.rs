use crate::{
    fixture::sequence::cue::{Cue, CueDataMode},
    lexer::token::Token,
    ui::{
        context::DemexUiContext,
        window::{edit::DemexEditWindow, DemexWindow},
    },
};

pub struct SequenceEditorTab<'a> {
    context: &'a mut DemexUiContext,
}

impl<'a> SequenceEditorTab<'a> {
    pub fn new(context: &'a mut DemexUiContext) -> Self {
        Self { context }
    }

    fn show_selected_sequence(&mut self, ui: &mut egui::Ui, sequence_id: u32) {
        let mut preset_handler = self.context.preset_handler.write();
        let sequence = preset_handler.get_sequence_mut(sequence_id);

        if let Ok(sequence) = sequence {
            ui.heading(sequence.name());

            egui_extras::TableBuilder::new(ui)
                .columns(egui_extras::Column::auto().at_least(20.0), 2)
                .column(egui_extras::Column::remainder().at_least(100.0))
                .columns(egui_extras::Column::auto().at_least(20.0), 8)
                .striped(true)
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("Cue Idx");
                    });

                    header.col(|ui| {
                        ui.heading("Builder");
                    });

                    header.col(|ui| {
                        ui.heading("Name");
                    });

                    header.col(|ui| {
                        ui.heading("In Delay");
                    });

                    header.col(|ui| {
                        ui.heading("In Fade");
                    });

                    header.col(|ui| {
                        ui.heading("Snap %");
                    });

                    header.col(|ui| {
                        ui.heading("Timing");
                    });

                    header.col(|ui| {
                        ui.heading("Trigger");
                    });

                    header.col(|ui| {
                        ui.heading("Block");
                    });

                    header.col(|ui| {
                        ui.heading("Fading");
                    });

                    header.col(|ui| {
                        ui.heading("MIB");
                    });
                })
                .body(|mut body| {
                    for cue in sequence.cues_mut() {
                        body.row(20.0, |mut row| {
                            let (cue_idx_major, cue_idx_minor) = cue.cue_idx();

                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{}.{}", cue_idx_major, cue_idx_minor));
                                    if ui.button("Sel").clicked() {
                                        self.context.command.extend_from_slice(&[
                                            Token::KeywordSequence,
                                            Token::Integer(sequence_id),
                                            Token::KeywordCue,
                                            Token::FloatingPoint(
                                                0.0,
                                                (cue_idx_major, cue_idx_minor),
                                            ),
                                        ]);
                                    }
                                });
                            });

                            row.col(|ui| match cue.data() {
                                CueDataMode::Builder(data) => {
                                    ui.label(format!("{} entries", data.len()));
                                    if ui.button("Edit").clicked() {
                                        self.context.window_handler.add_window(DemexWindow::Edit(
                                            DemexEditWindow::EditBuilderCue(
                                                sequence_id,
                                                cue.cue_idx(),
                                            ),
                                        ));
                                    }
                                }
                                CueDataMode::Default(_) => {
                                    ui.label("-");
                                }
                            });

                            row.col(|ui| {
                                egui_probe::Probe::new(cue.name_mut())
                                    .with_header("")
                                    .show(ui);
                            });

                            row.col(|ui| {
                                egui_probe::Probe::new(cue.in_delay_mut())
                                    .with_header("")
                                    .show(ui);
                            });

                            row.col(|ui| {
                                egui_probe::Probe::new(cue.in_fade_mut())
                                    .with_header("")
                                    .show(ui);
                            });

                            row.col(|ui| {
                                egui_probe::Probe::new(cue.snap_percent_mut())
                                    .with_header("")
                                    .show(ui);
                            });

                            row.col(|ui| {
                                egui_probe::Probe::new(cue.timing_mut())
                                    .with_header("")
                                    .show(ui);
                            });

                            row.col(|ui| {
                                egui_probe::Probe::new(cue.trigger_mut())
                                    .with_header("")
                                    .show(ui);
                            });

                            row.col(|ui| {
                                ui.checkbox(cue.block_mut(), "");
                            });

                            row.col(|ui| {
                                egui_probe::Probe::new(cue.fading_function_mut())
                                    .with_header("")
                                    .show(ui);
                            });

                            row.col(|ui| {
                                ui.checkbox(cue.move_in_black_mut(), "");
                            });
                        });
                    }
                });

            if ui.button("Add Builder Cue").clicked() {
                sequence.add_cue(Cue::new_default_builder(sequence.next_cue_idx()));
            }
        }
    }

    fn show_sequence_list(&mut self, ui: &mut egui::Ui) {
        let preset_handler = self.context.preset_handler.read();
        let sequences = preset_handler.sequences();

        for sequence in sequences.values() {
            if ui.button(sequence.name()).clicked() {
                self.context.global_sequence_select = Some(sequence.id());
            }
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        if let Some(sequence_id) = self.context.global_sequence_select {
            self.show_selected_sequence(ui, sequence_id);
        } else {
            self.show_sequence_list(ui);
        }
    }
}
