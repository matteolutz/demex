use color::{color_macro_ui, color_rgb_controls_ui};
use position::position_pan_tilt_controls_ui;

use crate::{
    fixture::channel2::feature::feature_type::FixtureFeatureType,
    parser::nodes::fixture_selector::FixtureSelectorContext,
};

pub mod color;
pub mod position;
// pub mod toggle_flags;

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut super::DemexUiContext) {
    let mut fixture_handler = context.fixture_handler.write();
    let preset_handler = context.preset_handler.read();

    if context.global_fixture_select.is_none() {
        ui.centered_and_justified(|ui| ui.label("No fixtures selected"));
        return;
    }

    let selected_fixtures = context
        .global_fixture_select
        .as_ref()
        .unwrap()
        .get_fixtures(
            &preset_handler,
            FixtureSelectorContext::new(&context.global_fixture_select),
        )
        .expect("fixture selection failed");

    let mut mutual_feature_types = fixture_handler
        .fixture_immut(selected_fixtures[0])
        .unwrap()
        .feature_types();

    for fixture_id in selected_fixtures.iter().skip(1) {
        let fixture_feature_types = fixture_handler
            .fixture_immut(*fixture_id)
            .unwrap()
            .feature_types();

        mutual_feature_types.retain(|feature_type| fixture_feature_types.contains(feature_type));
    }

    ui.style_mut().spacing.item_spacing = [0.0, 20.0].into();

    ui.heading("Fixture Controls");

    ui.horizontal(|ui| {
        ui.style_mut().spacing.item_spacing = [20.0, 10.0].into();

        for feature_type in mutual_feature_types {
            let is_channel_home = fixture_handler
                .fixture(selected_fixtures[0])
                .unwrap()
                .feature_display_state(feature_type)
                .map(|v| v.is_home())
                .unwrap_or(false);

            ui.vertical(|ui| {
                match feature_type {
                    FixtureFeatureType::ColorRGB => {
                        ui.set_width(100.0);

                        color_rgb_controls_ui(
                            ui,
                            is_channel_home,
                            &selected_fixtures,
                            &preset_handler,
                            &mut fixture_handler,
                        );
                    }
                    FixtureFeatureType::ColorMacro => {
                        ui.set_width(100.0);

                        color_macro_ui(
                            ui,
                            is_channel_home,
                            &selected_fixtures,
                            &mut fixture_handler,
                        );
                    }
                    FixtureFeatureType::PositionPanTilt => {
                        position_pan_tilt_controls_ui(
                            ui,
                            is_channel_home,
                            &selected_fixtures,
                            &preset_handler,
                            &mut fixture_handler,
                        );
                    }
                    unhandled_feature => {
                        ui.label(format!("todo: {:?}", unhandled_feature));
                    }
                }

                let home_button = ui.button("Home");
                if home_button.clicked() {
                    for fixture_id in selected_fixtures.iter() {
                        fixture_handler
                            .fixture(*fixture_id)
                            .unwrap()
                            .home_feature(feature_type)
                            .expect("");
                    }
                }
            });

            ui.separator();
        }
    });
}
