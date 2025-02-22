use state::SequenceEditorState;

use crate::{
    fixture::sequence::cue::{Cue, CueDataMode},
    ui::{
        context::DemexUiContext,
        window::{edit::DemexEditWindow, DemexWindow},
    },
};

mod state;

pub struct SequenceEditorTab<'a> {
    context: &'a mut DemexUiContext,
    id_source: egui::Id,
}

impl<'a> SequenceEditorTab<'a> {
    pub fn new(context: &'a mut DemexUiContext, name: &str) -> Self {
        Self {
            context,
            id_source: egui::Id::new(name),
        }
    }

    fn show_selected_sequence(
        &mut self,
        ui: &mut egui::Ui,
        sequence_id: u32,
        state: &mut SequenceEditorState,
    ) {
        let mut preset_handler = self.context.preset_handler.write();
        let sequence = preset_handler.get_sequence_mut(sequence_id);

        if ui.button("Back").clicked() {
            state.selected_sequence = None;
        }

        if let Ok(sequence) = sequence {
            ui.heading(sequence.name());

            egui_extras::TableBuilder::new(ui)
                .columns(egui_extras::Column::auto(), 2)
                .column(egui_extras::Column::remainder())
                .columns(egui_extras::Column::auto(), 7)
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
                        ui.heading("Out Delay");
                    });

                    header.col(|ui| {
                        ui.heading("Out Fade");
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
                })
                .body(|mut body| {
                    for cue in sequence.cues_mut() {
                        body.row(20.0, |mut row| {
                            let (cue_idx_major, cue_idx_minor) = cue.cue_idx();

                            row.col(|ui| {
                                ui.label(format!("{}.{}", cue_idx_major, cue_idx_minor));
                            });

                            row.col(|ui| match cue.data() {
                                CueDataMode::Builder(data) => {
                                    ui.label(format!("{} entries", data.len()));
                                    if ui.button("Edit").clicked() {
                                        self.context.windows.push(DemexWindow::Edit(
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
                                egui_probe::Probe::new(cue.out_delay_mut())
                                    .with_header("")
                                    .show(ui);
                            });

                            row.col(|ui| {
                                egui_probe::Probe::new(cue.out_fade_mut())
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
                        });
                    }
                });

            if ui.button("Add Builder Cue").clicked() {
                sequence.add_cue(Cue::new_default_builder(sequence.next_cue_idx()));
            }
        }
    }

    fn show_sequence_list(&self, ui: &mut egui::Ui, state: &mut SequenceEditorState) {
        let preset_handler = self.context.preset_handler.read();
        let sequences = preset_handler.sequences();

        for sequence in sequences.values() {
            if ui.button(sequence.name()).clicked() {
                state.selected_sequence = Some(sequence.id());
            }
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let id = ui.make_persistent_id(self.id_source);
        let mut state = ui
            .ctx()
            .data_mut(|d| d.get_persisted::<SequenceEditorState>(id))
            .unwrap_or_default();

        if let Some(sequence_id) = state.selected_sequence {
            self.show_selected_sequence(ui, sequence_id, &mut state);
        } else {
            self.show_sequence_list(ui, &mut state);
        }

        ui.ctx().data_mut(|d| {
            d.insert_persisted(id, state);
        });
    }
}
