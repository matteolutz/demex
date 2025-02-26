use color::{color_macro_ui, color_rgb_controls_ui};
use itertools::Itertools;
use position::position_pan_tilt_controls_ui;
use slider::feature_f32_slider;
use toggle_flags::toggle_flags_controls_ui;

use crate::fixture::channel2::feature::{
    feature_type::FixtureFeatureType, feature_value::FixtureFeatureValue,
};

pub mod color;
pub mod position;
pub mod slider;
pub mod toggle_flags;

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
        .map(|selection| selection.fixtures())
        .unwrap_or_default();

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

    ui.heading(format!("Fixture Controls - {:?}", selected_fixtures));

    ui.vertical(|ui| {
        ui.style_mut().spacing.item_spacing = [20.0, 10.0].into();

        for (_, feature_group) in preset_handler
            .feature_groups()
            .iter()
            .sorted_by_key(|(id, _)| *id)
        {
            ui.vertical(|ui| {
                ui.heading(feature_group.name());

                ui.horizontal(|ui| {
                    ui.add_space(30.0);

                    for feature_type in mutual_feature_types
                        .iter()
                        .filter(|feature_type| feature_group.feature_types().contains(feature_type))
                    {
                        let is_channel_home = fixture_handler
                            .fixture(selected_fixtures[0])
                            .unwrap()
                            .feature_is_home_programmer(*feature_type)
                            .unwrap_or(false);

                        ui.vertical(|ui| {
                            match feature_type {
                                FixtureFeatureType::SingleValue { channel_type } => {
                                    feature_f32_slider(
                                        ui,
                                        is_channel_home,
                                        selected_fixtures,
                                        FixtureFeatureType::SingleValue {
                                            channel_type: *channel_type,
                                        },
                                        &mut fixture_handler,
                                        &preset_handler,
                                        |value| {
                                            if let FixtureFeatureValue::SingleValue {
                                                value, ..
                                            } = value
                                            {
                                                Some(value)
                                            } else {
                                                None
                                            }
                                        },
                                        |value| FixtureFeatureValue::SingleValue {
                                            value,
                                            channel_type: *channel_type,
                                        },
                                    );
                                }

                                FixtureFeatureType::ColorRGB => {
                                    ui.set_width(100.0);

                                    color_rgb_controls_ui(
                                        ui,
                                        is_channel_home,
                                        selected_fixtures,
                                        &preset_handler,
                                        &mut fixture_handler,
                                    );
                                }
                                FixtureFeatureType::ColorWheel => {
                                    ui.set_width(100.0);

                                    color_macro_ui(
                                        ui,
                                        is_channel_home,
                                        selected_fixtures,
                                        &mut fixture_handler,
                                    );
                                }
                                FixtureFeatureType::PositionPanTilt => {
                                    position_pan_tilt_controls_ui(
                                        ui,
                                        is_channel_home,
                                        selected_fixtures,
                                        &preset_handler,
                                        &mut fixture_handler,
                                    );
                                }
                                FixtureFeatureType::ToggleFlags => {
                                    toggle_flags_controls_ui(
                                        ui,
                                        is_channel_home,
                                        selected_fixtures,
                                        &preset_handler,
                                        &mut fixture_handler,
                                    );
                                }
                            }

                            let home_button = ui.button("Home");
                            if home_button.clicked() {
                                for fixture_id in selected_fixtures.iter() {
                                    fixture_handler
                                        .fixture(*fixture_id)
                                        .unwrap()
                                        .home_feature(*feature_type)
                                        .expect("");
                                }
                            }
                        });
                    }
                });
            });

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(20.0);
        }
    });
}
