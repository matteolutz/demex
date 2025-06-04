use strum::IntoEnumIterator;

use crate::{
    fixture::{
        channel3::{
            attribute::FixtureChannel3Attribute,
            feature::feature_group::FixtureChannel3FeatureGroup,
        },
        effect::feature::runtime::FeatureEffectRuntime,
        effect2::effect::Effect2Part,
    },
    ui::{
        components::{
            button::{icon::DemexIcon, icon_button},
            wave_editor::WaveEditor,
        },
        utils::vec::with_index_mut,
    },
};

pub fn edit_effect_ui(
    ui: &mut egui::Ui,
    top_level_id: String,
    effect_runtime: &mut FeatureEffectRuntime,
    feature_group: FixtureChannel3FeatureGroup,
) {
    ui.vertical(|ui| {
        ui.heading("Effect");

        egui_probe::Probe::new(effect_runtime.phase_mut())
            .with_header("Phase")
            .show(ui);
        egui_probe::Probe::new(effect_runtime.speed_mut())
            .with_header("Speed")
            .show(ui);

        ui.add_space(20.0);

        effect_runtime
            .effect_mut()
            .parts_mut()
            .retain_mut(with_index_mut(|idx, part: &mut Effect2Part| {
                let mut retain = true;

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("Part {}", idx + 1));
                        if icon_button(ui, DemexIcon::Delete).clicked() {
                            retain = false;
                        }
                    });

                    ui.horizontal(|ui| {
                        WaveEditor::new(
                            format!("{}EffectPart{}WaveEditor", top_level_id, idx),
                            part.wave_mut(),
                        )
                        .show(ui);

                        ui.vertical(|ui| {
                            egui::ScrollArea::vertical()
                                .id_salt(format!("{}EffectPart{}Attributes", top_level_id, idx))
                                .auto_shrink(emath::Vec2b::new(false, true))
                                .max_width(ui.available_width())
                                .max_height(300.0)
                                .show(ui, |ui| {
                                    ui.vertical(|ui| {
                                        for attribute in
                                            FixtureChannel3Attribute::iter().filter(|attribute| {
                                                feature_group == FixtureChannel3FeatureGroup::All
                                                    || attribute.feature().feature_group()
                                                        == feature_group
                                            })
                                        {
                                            let attribute_string = attribute.to_string();
                                            let was_selected =
                                                part.attributes().contains(&attribute_string);
                                            let mut is_selected = was_selected;

                                            ui.checkbox(&mut is_selected, attribute_string.clone());

                                            if is_selected != was_selected {
                                                if is_selected {
                                                    part.attributes_mut().push(attribute_string);
                                                } else {
                                                    part.attributes_mut().retain(|attribute| {
                                                        attribute != &attribute_string
                                                    });
                                                }
                                            }
                                        }
                                    });
                                });

                            ui.add_space(10.0);

                            egui_probe::Probe::new(part.phase_offset_mut())
                                .with_header("Phase offset")
                                .show(ui);

                            egui_probe::Probe::new(part.phase_multiplier_mut())
                                .with_header("Phase multiplier")
                                .show(ui);
                        });
                    });
                });

                retain
            }));

        if ui.button("Add part").clicked() {
            effect_runtime
                .effect_mut()
                .parts_mut()
                .push(Effect2Part::default());
        }
    });
}
