use egui::color_picker::color_edit_button_rgb;
use itertools::Itertools;

use crate::{
    fixture::{
        channel2::{
            color::color_gel::ColorGelTrait,
            feature::{
                feature_config::FixtureFeatureConfig, feature_type::FixtureFeatureType,
                feature_value::FixtureFeatureValue,
            },
        },
        handler::FixtureHandler,
        presets::PresetHandler,
    },
    ui::constants::PLEASE_SELECT_FIXTURES_OF_SAME_TYPE_AND_MODE,
};

pub fn color_macro_ui(
    ui: &mut egui::Ui,
    is_channel_home: bool,
    selected_fixtures: &[u32],
    fixture_handler: &mut FixtureHandler,
) {
    ui.vertical(|ui| {
        ui.label(
            egui::RichText::from("Color Macro").color(if is_channel_home {
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

        if let Some(FixtureFeatureConfig::ColorMacro { macros }) = fixture_handler
            .fixture_immut(selected_fixtures[0])
            .unwrap()
            .feature_config_by_type(FixtureFeatureType::ColorMacro)
            .cloned()
        {
            for (macro_idx, (_, macro_color)) in macros.iter().enumerate() {
                ui.scope(|ui| {
                    let macro_color_rgb = macro_color.get_rgb();
                    let egui_color = egui::Color32::from_rgb(
                        (macro_color_rgb[0] * 255.0) as u8,
                        (macro_color_rgb[1] * 255.0) as u8,
                        (macro_color_rgb[2] * 255.0) as u8,
                    );

                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = egui_color;
                    ui.style_mut().visuals.widgets.hovered.weak_bg_fill = egui_color;
                    ui.style_mut().visuals.widgets.active.weak_bg_fill = egui_color;

                    ui.horizontal(|ui| {
                        let color_button = ui.button("     ");

                        if color_button.clicked() {
                            for fixture_id in selected_fixtures.iter() {
                                fixture_handler
                                    .fixture(*fixture_id)
                                    .unwrap()
                                    .set_feature_value(FixtureFeatureValue::ColorMacro {
                                        macro_idx,
                                    })
                                    .unwrap();
                            }
                        }

                        ui.label(macro_color.to_string())
                    });
                });
            }
        }
    });
}

pub fn color_rgb_controls_ui(
    ui: &mut eframe::egui::Ui,
    is_channel_home: bool,
    selected_fixtures: &[u32],
    preset_handler: &PresetHandler,
    fixture_handler: &mut FixtureHandler,
) {
    ui.vertical(|ui| {
        ui.label(egui::RichText::from("Color RGB").color(if is_channel_home {
            egui::Color32::PLACEHOLDER
        } else {
            egui::Color32::YELLOW
        }));

        let colors = selected_fixtures
            .iter()
            .map(|f_id| {
                fixture_handler
                    .fixture(*f_id)
                    .unwrap()
                    .feature_value_programmer(FixtureFeatureType::ColorRGB, preset_handler)
                    .unwrap()
            })
            .collect::<Vec<_>>();

        if let FixtureFeatureValue::ColorRGB { r, g, b } = colors[0] {
            let mut color = [r, g, b];

            ui.style_mut().spacing.item_spacing = [10.0, 0.0].into();
            color_edit_button_rgb(ui, &mut color);

            if color != [r, g, b] {
                for fixture_id in selected_fixtures.iter() {
                    fixture_handler
                        .fixture(*fixture_id)
                        .unwrap()
                        .set_feature_value(FixtureFeatureValue::ColorRGB {
                            r: color[0],
                            g: color[1],
                            b: color[2],
                        })
                        .expect("");
                }
            }
        }
    });
}
