use egui::color_picker::color_picker_color32;

use crate::fixture::{
    channel2::feature::{
        feature_config::FixtureFeatureConfig, feature_type::FixtureFeatureType,
        feature_value::FixtureFeatureValue,
    },
    handler::FixtureHandler,
    presets::PresetHandler,
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

        if let Some(FixtureFeatureConfig::ColorMacro { macros }) = fixture_handler
            .fixture_immut(selected_fixtures[0])
            .unwrap()
            .feature_config_by_type(FixtureFeatureType::ColorMacro)
            .cloned()
        {
            for (macro_val, macro_color) in macros.iter() {
                ui.scope(|ui| {
                    let egui_color = egui::Color32::from_rgb(
                        (macro_color[0] * 255.0) as u8,
                        (macro_color[1] * 255.0) as u8,
                        (macro_color[2] * 255.0) as u8,
                    );

                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = egui_color;
                    ui.style_mut().visuals.widgets.hovered.weak_bg_fill = egui_color;
                    ui.style_mut().visuals.widgets.active.weak_bg_fill = egui_color;

                    let color_button = ui.button("     ");

                    if color_button.clicked() {
                        for fixture_id in selected_fixtures.iter() {
                            fixture_handler
                                .fixture(*fixture_id)
                                .unwrap()
                                .set_feature_value(FixtureFeatureValue::ColorMacro {
                                    macro_val: *macro_val,
                                })
                                .unwrap();
                        }
                    }
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
            let mut color = eframe::egui::Color32::from_rgb(
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8,
            );

            ui.style_mut().spacing.item_spacing = [10.0, 0.0].into();
            if color_picker_color32(ui, &mut color, egui::color_picker::Alpha::Opaque) {
                for fixture_id in selected_fixtures.iter() {
                    fixture_handler
                        .fixture(*fixture_id)
                        .unwrap()
                        .set_feature_value(FixtureFeatureValue::ColorRGB {
                            r: color.r() as f32 / 255.0,
                            g: color.g() as f32 / 255.0,
                            b: color.b() as f32 / 255.0,
                        })
                        .expect("");
                }
            }
        }
    });
}
