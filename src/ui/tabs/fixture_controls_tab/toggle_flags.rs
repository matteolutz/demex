use egui::RichText;
use itertools::Itertools;

use crate::{
    fixture::{
        channel2::feature::{
            feature_config::FixtureFeatureConfig, feature_type::FixtureFeatureType,
            feature_value::FixtureFeatureValue,
        },
        handler::FixtureHandler,
        presets::PresetHandler,
    },
    ui::constants::PLEASE_SELECT_FIXTURES_OF_SAME_TYPE_AND_MODE,
};

pub fn toggle_flags_controls_ui(
    ui: &mut eframe::egui::Ui,
    is_channel_home: bool,
    selected_fixtures: &[u32],
    preset_handler: &PresetHandler,
    fixture_handler: &mut FixtureHandler,
) {
    ui.vertical(|ui| {
        ui.set_width(100.0);
        ui.label(
            egui::RichText::from("Toggle Flags").color(if is_channel_home {
                egui::Color32::PLACEHOLDER
            } else {
                egui::Color32::YELLOW
            }),
        );

        if selected_fixtures
            .iter()
            .map(|f_id| fixture_handler.fixture_immut(*f_id).unwrap().fixture_type())
            .unique()
            .count()
            > 1
        {
            ui.colored_label(
                egui::Color32::YELLOW,
                PLEASE_SELECT_FIXTURES_OF_SAME_TYPE_AND_MODE,
            );
            return;
        }

        ui.horizontal(|ui| {
            if let FixtureFeatureConfig::ToggleFlags { toggle_flags } = fixture_handler
                .fixture_immut(selected_fixtures[0])
                .unwrap()
                .feature_config_by_type(FixtureFeatureType::ToggleFlags)
                .cloned()
                .unwrap()
            {
                if let FixtureFeatureValue::ToggleFlags { set_flags } = fixture_handler
                    .fixture_immut(selected_fixtures[0])
                    .unwrap()
                    .feature_value_programmer(FixtureFeatureType::ToggleFlags, preset_handler)
                    .unwrap()
                {
                    for (idx, flags) in toggle_flags.iter().enumerate() {
                        let set_flag = &set_flags[idx];

                        ui.collapsing(
                            format!(
                                "{}. Flag{}",
                                idx + 1,
                                if set_flag.is_none() { " (unset)" } else { "" }
                            ),
                            |ui| {
                                for (flag_name, _) in
                                    flags.iter().sorted_by_key(|(_, value)| **value)
                                {
                                    let flag_button = ui.button(RichText::new(flag_name).color(
                                        if set_flag.is_some()
                                            && set_flag.as_ref().unwrap() == flag_name
                                        {
                                            egui::Color32::YELLOW
                                        } else {
                                            egui::Color32::PLACEHOLDER
                                        },
                                    ));

                                    if flag_button.clicked() {
                                        let mut new_flags = set_flags.clone();
                                        new_flags[idx] = Some(flag_name.clone());

                                        for fixture_id in selected_fixtures {
                                            fixture_handler
                                                .fixture(*fixture_id)
                                                .unwrap()
                                                .set_feature_value(
                                                    FixtureFeatureValue::ToggleFlags {
                                                        set_flags: new_flags.clone(),
                                                    },
                                                )
                                                .unwrap();
                                        }
                                    }
                                }

                                ui.separator();

                                if ui.button(format!("Unset flag {}", idx + 1)).clicked() {
                                    let mut new_flags = set_flags.clone();
                                    new_flags[idx] = None;

                                    for fixture_id in selected_fixtures {
                                        fixture_handler
                                            .fixture(*fixture_id)
                                            .unwrap()
                                            .set_feature_value(FixtureFeatureValue::ToggleFlags {
                                                set_flags: new_flags.clone(),
                                            })
                                            .unwrap();
                                    }
                                }
                            },
                        );
                    }
                }
            }
        });
    });
}
