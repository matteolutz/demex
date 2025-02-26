use itertools::Itertools;

use crate::{
    fixture::{
        channel2::feature::{
            feature_config::FixtureFeatureConfig, feature_type::FixtureFeatureType,
            feature_value::FixtureFeatureValue, wheel::WheelFeatureValue,
        },
        handler::FixtureHandler,
    },
    ui::constants::PLEASE_SELECT_FIXTURES_OF_SAME_TYPE_AND_MODE,
};

pub fn gobo_wheel_ui(
    ui: &mut egui::Ui,
    is_channel_home: bool,
    selected_fixtures: &[u32],
    fixture_handler: &mut FixtureHandler,
) {
    ui.vertical(|ui| {
        ui.label(egui::RichText::from("Gobo").color(if is_channel_home {
            egui::Color32::PLACEHOLDER
        } else {
            egui::Color32::YELLOW
        }));

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

        if let Some(FixtureFeatureConfig::GoboWheel { wheel_config }) = fixture_handler
            .fixture_immut(selected_fixtures[0])
            .unwrap()
            .feature_config_by_type(FixtureFeatureType::GoboWheel)
            .cloned()
        {
            for (macro_idx, (_, gobo_macro)) in wheel_config.macros().enumerate() {
                ui.scope(|ui| {
                    ui.horizontal(|ui| {
                        if let Some(gobo_image) = gobo_macro.image() {
                            let uri = format!(
                                "file://{}",
                                std::fs::canonicalize(gobo_image)
                                    .ok()
                                    .and_then(|p| p.to_str().map(|s| s.to_string()))
                                    .unwrap()
                            );

                            ui.add(
                                egui::Image::new(egui::ImageSource::Uri(uri.into()))
                                    .max_width(200.0),
                            );
                        }

                        let gobo_button = ui.button("     ");

                        if gobo_button.clicked() {
                            for fixture_id in selected_fixtures.iter() {
                                fixture_handler
                                    .fixture(*fixture_id)
                                    .unwrap()
                                    .set_feature_value(FixtureFeatureValue::GoboWheel {
                                        wheel_value: WheelFeatureValue::Macro(macro_idx),
                                    })
                                    .unwrap();
                            }
                        }

                        ui.label(gobo_macro.name())
                    });
                });
            }
        }
    });
}
