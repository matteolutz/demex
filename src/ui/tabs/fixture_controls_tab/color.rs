use egui::color_picker::color_picker_color32;

use crate::fixture::{
    channel2::feature::{feature_type::FixtureFeatureType, feature_value::FixtureFeatureValue},
    handler::FixtureHandler,
    presets::PresetHandler,
};

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
