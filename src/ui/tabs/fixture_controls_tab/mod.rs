use strum::IntoEnumIterator;

use crate::fixture::channel3::{
    channel_value::FixtureChannelValue3, feature::feature_group::FixtureChannel3FeatureGroup,
};

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut super::DemexUiContext) {
    let mut fixture_handler = context.fixture_handler.write();
    let preset_handler = context.preset_handler.read();
    let timing_handler = context.timing_handler.read();
    let patch = context.patch.read();

    if context.global_fixture_select.is_none() {
        ui.centered_and_justified(|ui| ui.label("No fixtures selected"));
        return;
    }

    let selected_fixtures = context
        .global_fixture_select
        .as_ref()
        .map(|selection| selection.fixtures())
        .unwrap_or_default();

    ui.style_mut().spacing.item_spacing = [0.0, 20.0].into();

    ui.heading(format!("Fixture Controls - {:?}", selected_fixtures));

    ui.vertical(|ui| {
        ui.style_mut().spacing.item_spacing = [20.0, 10.0].into();

        for feature_group in FixtureChannel3FeatureGroup::iter() {
            ui.vertical(|ui| {
                ui.heading(feature_group.name());

                ui.horizontal(|ui| {
                    ui.add_space(30.0);

                    for channel_name in fixture_handler
                        .fixture_immut(selected_fixtures[0])
                        .unwrap()
                        .get_channels_in_feature_group(patch.fixture_types(), feature_group)
                        .unwrap()
                    {
                        ui.vertical(|ui| {
                            let programmer_value = fixture_handler
                                .fixture_immut(selected_fixtures[0])
                                .unwrap()
                                .get_programmer_value(&channel_name)
                                .unwrap()
                                .clone();

                            let is_channel_home = programmer_value.is_home();

                            let (channel_function_idx, f_value) = programmer_value.get_as_discrete(
                                fixture_handler.fixture_immut(selected_fixtures[0]).unwrap(),
                                patch.fixture_types(),
                                &channel_name,
                                &preset_handler,
                                &timing_handler,
                            );

                            ui.vertical(|ui| {
                                ui.set_width(100.0);

                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::from(format!("{}", channel_name)).color(
                                            if is_channel_home {
                                                egui::Color32::PLACEHOLDER
                                            } else {
                                                egui::Color32::YELLOW
                                            },
                                        ),
                                    );

                                    let mut slider_val = f_value;

                                    ui.add(
                                        egui::Slider::new(&mut slider_val, 0.0..=1.0).vertical(),
                                    );

                                    if slider_val != f_value {
                                        for fixture in selected_fixtures {
                                            fixture_handler
                                                .fixture(*fixture)
                                                .unwrap()
                                                .set_programmer_value(
                                                    patch.fixture_types(),
                                                    &channel_name,
                                                    FixtureChannelValue3::Discrete {
                                                        channel_function_idx,
                                                        value: slider_val,
                                                    },
                                                )
                                                .unwrap();
                                        }
                                    }
                                });
                            });

                            ui.vertical(|ui| {
                                let home_button = ui.button("Home");
                                if home_button.clicked() {
                                    for fixture_id in selected_fixtures.iter() {
                                        fixture_handler
                                            .fixture(*fixture_id)
                                            .unwrap()
                                            .set_programmer_value(
                                                patch.fixture_types(),
                                                &channel_name,
                                                FixtureChannelValue3::Home,
                                            )
                                            .expect("");
                                    }
                                }
                            });
                        });

                        ui.separator();
                    }
                });
            });

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(20.0);
        }
    });
}
