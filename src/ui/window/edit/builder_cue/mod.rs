use crate::{
    fixture::{
        presets::preset::FixturePresetId,
        sequence::cue::{Cue, CueBuilderEntry, CueDataMode},
    },
    ui::utils::vec::with_index_mut,
};

pub struct DisplayEntry {
    pub id: u32,
    pub name: String,
}

pub struct PresetDisplayEntry {
    pub id: FixturePresetId,
    pub name: String,
}

const NOT_SELECTED_LABEL: &str = "Not selected";

pub fn edit_builder_cue_ui(
    ui: &mut egui::Ui,
    sequence_id: u32,
    cue: &mut Cue,
    groups: Vec<DisplayEntry>,
    presets: Vec<PresetDisplayEntry>,
) {
    let cue_idx = cue.cue_idx();

    if let CueDataMode::Builder(data) = cue.data_mut() {
        egui_extras::TableBuilder::new(ui)
            .striped(true)
            .column(egui_extras::Column::auto())
            .columns(egui_extras::Column::auto().at_least(200.0), 2)
            .column(egui_extras::Column::auto())
            .header(20.0, |mut ui| {
                ui.col(|ui| {
                    ui.heading("Idx");
                });

                ui.col(|ui| {
                    ui.heading("Fixture Group");
                });

                ui.col(|ui| {
                    ui.heading("Preset");
                });

                ui.col(|_| {});
            })
            .body(|mut ui| {
                data.retain_mut(with_index_mut(|idx, entry: &mut CueBuilderEntry| {
                    let mut retain = true;

                    ui.row(20.0, |mut ui| {
                        ui.col(|ui| {
                            ui.label((idx + 1).to_string());
                        });

                        ui.col(|ui| {
                            egui::ComboBox::from_id_salt(format!(
                                "{} - {:?} - GroupSelector",
                                sequence_id, cue_idx
                            ))
                            .selected_text(
                                entry
                                    .group_id
                                    .and_then(|group_id| {
                                        groups
                                            .iter()
                                            .find(|g| g.id == group_id)
                                            .map(|g| g.name.as_str())
                                    })
                                    .unwrap_or(NOT_SELECTED_LABEL),
                            )
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut entry.group_id, None, NOT_SELECTED_LABEL);

                                for group in &groups {
                                    ui.selectable_value(
                                        &mut entry.group_id,
                                        Some(group.id),
                                        group.name.clone(),
                                    );
                                }
                            });
                        });

                        ui.col(|ui| {
                            egui::ComboBox::from_id_salt(format!(
                                "{} - {:?} - PresetSelector",
                                sequence_id, cue_idx
                            ))
                            .selected_text(
                                entry
                                    .preset_id
                                    .and_then(|preset_id| {
                                        presets
                                            .iter()
                                            .find(|p| p.id == preset_id)
                                            .map(|p| p.name.as_str())
                                    })
                                    .unwrap_or(NOT_SELECTED_LABEL),
                            )
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut entry.preset_id, None, NOT_SELECTED_LABEL);

                                for preset in &presets {
                                    ui.selectable_value(
                                        &mut entry.preset_id,
                                        Some(preset.id),
                                        preset.name.clone(),
                                    );
                                }
                            });
                        });

                        ui.col(|ui| {
                            if ui.button("-").clicked() {
                                retain = false;
                            }
                        });
                    });

                    retain
                }));
            });

        if ui.button("+").clicked() {
            data.push(CueBuilderEntry::default());
        }

        ui.add_space(20.0);
    } else {
        ui.colored_label(egui::Color32::LIGHT_RED, "Error: Cue is not a builder cue.");
    }
}
