use egui::color_picker::color_picker_color32;

use crate::{
    fixture::{
        self,
        channel::{
            value::{FixtureChannelDiscreteValue, FixtureChannelValue, FixtureChannelValueTrait},
            FIXTURE_CHANNEL_COLOR_ID, FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
        },
    },
    parser::nodes::fixture_selector::FixtureSelectorContext,
    ui::components::position_selector::PositionSelector,
};

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut super::DemexUiContext) {
    if context.global_fixture_select.is_none() {
        ui.centered_and_justified(|ui| ui.label("No fixtures selected"));
        return;
    }

    let selected_fixtures = context
        .global_fixture_select
        .as_ref()
        .unwrap()
        .get_fixtures(
            &context.preset_handler,
            FixtureSelectorContext::new(&context.global_fixture_select),
        )
        .expect("fixture selection failed");

    if selected_fixtures.is_empty() {
        context.global_fixture_select = None;
        return;
    }

    let mut mutual_channel_types = context
        .fixture_handler
        .fixture(selected_fixtures[0])
        .unwrap()
        .channel_types()
        .clone();

    for fixture_id in selected_fixtures.iter().skip(1) {
        let fixture_channel_types = context
            .fixture_handler
            .fixture(*fixture_id)
            .unwrap()
            .channel_types();

        mutual_channel_types.retain(|channel_type| fixture_channel_types.contains(channel_type));
    }

    ui.style_mut().spacing.item_spacing = [0.0, 20.0].into();

    ui.heading("Fixture Control");

    ui.horizontal(|ui| {
        ui.style_mut().spacing.item_spacing = [20.0, 10.0].into();

        for channel_type in mutual_channel_types {
            let channel_name = context
                .fixture_handler
                .fixture(selected_fixtures[0])
                .unwrap()
                .channel_name(channel_type)
                .unwrap();

            let is_channel_home = context
                .fixture_handler
                .fixture(selected_fixtures[0])
                .unwrap()
                .channel_value_programmer(channel_type)
                .map(|v| v.is_home())
                .unwrap_or(false);

            ui.vertical(|ui| {
                match channel_type {
                    fixture::channel::FIXTURE_CHANNEL_INTENSITY_ID
                    | fixture::channel::FIXTURE_CHANNEL_STROBE
                    | fixture::channel::FIXTURE_CHANNEL_ZOOM => {
                        ui.vertical(|ui| {
                            ui.set_width(100.0);
                            ui.label(egui::RichText::from(channel_name).color(
                                if is_channel_home {
                                    egui::Color32::PLACEHOLDER
                                } else {
                                    egui::Color32::YELLOW
                                },
                            ));

                            ui.add(
                                eframe::egui::Slider::from_get_set(0.0..=1.0, |val| {
                                    if let Some(val) = val {
                                        for fixture_id in selected_fixtures.iter() {
                                            context
                                                .fixture_handler
                                                .fixture(*fixture_id)
                                                .unwrap()
                                                .set_channel_single_value(channel_type, val as f32)
                                                .expect("");
                                        }

                                        val
                                    } else {
                                        context
                                            .fixture_handler
                                            .fixture(selected_fixtures[0])
                                            .unwrap()
                                            .channel_single_value_programmer(
                                                channel_type,
                                                &context.preset_handler,
                                            )
                                            .expect("")
                                            as f64
                                    }
                                })
                                .vertical(),
                            );
                        });
                    }
                    fixture::channel::FIXTURE_CHANNEL_COLOR_ID => {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::from(channel_name).color(
                                if is_channel_home {
                                    egui::Color32::PLACEHOLDER
                                } else {
                                    egui::Color32::YELLOW
                                },
                            ));

                            let fixture_color = context
                                .fixture_handler
                                .fixture(selected_fixtures[0])
                                .unwrap()
                                .color_programmer()
                                .expect("");

                            let rgb_color = fixture_color
                                // TODO: change this, to use the corresponding fixture
                                .as_quadruple(
                                    &context.preset_handler,
                                    selected_fixtures[0],
                                    FIXTURE_CHANNEL_COLOR_ID,
                                )
                                .map(|val| [val[0], val[1], val[2]])
                                .expect("todo: rgb color error handling");

                            let mut color = eframe::egui::Color32::from_rgb(
                                (rgb_color[0] * 255.0) as u8,
                                (rgb_color[1] * 255.0) as u8,
                                (rgb_color[2] * 255.0) as u8,
                            );

                            ui.style_mut().spacing.item_spacing = [10.0, 0.0].into();
                            if color_picker_color32(
                                ui,
                                &mut color,
                                egui::color_picker::Alpha::Opaque,
                            ) {
                                for fixture_id in selected_fixtures.iter() {
                                    context
                                        .fixture_handler
                                        .fixture(*fixture_id)
                                        .unwrap()
                                        .set_color(FixtureChannelValue::Discrete(
                                            FixtureChannelDiscreteValue::Quadruple([
                                                color.r() as f32 / 255.0,
                                                color.g() as f32 / 255.0,
                                                color.b() as f32 / 255.0,
                                                0.0,
                                            ]),
                                        ))
                                        .expect("");
                                }
                            }
                        });
                    }
                    fixture::channel::FIXTURE_CHANNEL_POSITION_PAN_TILT_ID => {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::from(channel_name).color(
                                if is_channel_home {
                                    egui::Color32::PLACEHOLDER
                                } else {
                                    egui::Color32::YELLOW
                                },
                            ));

                            ui.add(PositionSelector::new(|val| {
                                if let Some(val) = val {
                                    for fixture_id in selected_fixtures.iter() {
                                        context
                                            .fixture_handler
                                            .fixture(*fixture_id)
                                            .unwrap()
                                            .set_position_pan_tilt(FixtureChannelValue::Discrete(
                                                FixtureChannelDiscreteValue::Pair(val.into()),
                                            ))
                                            .expect("");
                                    }

                                    Some(eframe::egui::vec2(0.0, 0.0))
                                } else {
                                    let pos_val = context
                                        .fixture_handler
                                        .fixture(selected_fixtures[0])
                                        .unwrap()
                                        .position_pan_tilt_programmer()
                                        .expect("");

                                    let pos = pos_val
                                        // TODO: change this, to use the corresponding fixture
                                        .as_pair(
                                            &context.preset_handler,
                                            selected_fixtures[0],
                                            FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
                                        )
                                        .expect("");

                                    Some(eframe::egui::vec2(pos[0], pos[1]))
                                }
                            }));
                        });
                    }
                    fixture::channel::FIXTURE_CHANNEL_TOGGLE_FLAGS => {
                        ui.style_mut().spacing.item_spacing = [0.0, 10.0].into();
                        ui.style_mut().wrap_mode = Some(eframe::egui::TextWrapMode::Extend);

                        ui.vertical(|ui| {
                            ui.set_width(100.0);
                            ui.label(egui::RichText::from(channel_name).color(
                                if is_channel_home {
                                    egui::Color32::PLACEHOLDER
                                } else {
                                    egui::Color32::YELLOW
                                },
                            ));

                            let unset_button = ui.button("Unset");
                            if unset_button.clicked() {
                                for fixture_id in selected_fixtures.iter() {
                                    context
                                        .fixture_handler
                                        .fixture(*fixture_id)
                                        .unwrap()
                                        .unset_toggle_flags()
                                        .expect("");
                                }
                            }

                            ui.separator();

                            let mut mutual_flags = context
                                .fixture_handler
                                .fixture(selected_fixtures[0])
                                .unwrap()
                                .toggle_flags();
                            for f in selected_fixtures.iter().skip(1) {
                                mutual_flags.retain(|flag| {
                                    context
                                        .fixture_handler
                                        .fixture(*f)
                                        .unwrap()
                                        .toggle_flags()
                                        .contains(flag)
                                });
                            }

                            for flag in mutual_flags {
                                let flag_button = ui.button(flag.clone());

                                if flag_button.clicked() {
                                    for fixture_id in selected_fixtures.iter() {
                                        context
                                            .fixture_handler
                                            .fixture(*fixture_id)
                                            .unwrap()
                                            .set_toggle_flag(&flag)
                                            .expect("");
                                    }
                                }
                            }
                        });
                    }
                    _ => {
                        ui.vertical(|ui| {
                            ui.set_width(100.0);

                            ui.label(egui::RichText::from(channel_name).color(
                                if is_channel_home {
                                    egui::Color32::PLACEHOLDER
                                } else {
                                    egui::Color32::YELLOW
                                },
                            ));

                            ui.label(egui::RichText::from(channel_type.to_string()).small());

                            ui.add(
                                eframe::egui::Slider::from_get_set(0.0..=1.0, |val| {
                                    if let Some(val) = val {
                                        for fixture_id in selected_fixtures.iter() {
                                            context
                                                .fixture_handler
                                                .fixture(*fixture_id)
                                                .unwrap()
                                                .set_channel_single_value(channel_type, val as f32)
                                                .expect("");
                                        }

                                        val
                                    } else {
                                        context
                                            .fixture_handler
                                            .fixture(selected_fixtures[0])
                                            .unwrap()
                                            .channel_single_value_programmer(
                                                channel_type,
                                                &context.preset_handler,
                                            )
                                            .expect("")
                                            as f64
                                    }
                                })
                                .vertical(),
                            );
                        });
                    }
                }

                let home_button = ui.button("Home");
                if home_button.clicked() {
                    for fixture_id in selected_fixtures.iter() {
                        context
                            .fixture_handler
                            .fixture(*fixture_id)
                            .unwrap()
                            .set_channel_value(channel_type, FixtureChannelValue::any_home())
                            .expect("");
                    }
                }
            });

            ui.separator();
        }
    });
}
