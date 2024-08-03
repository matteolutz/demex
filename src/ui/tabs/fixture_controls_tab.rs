use crate::{
    fixture::{self, channel::FixtureChannel},
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
        .get_fixtures(&context.preset_handler)
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

    ui.heading("fixture control");

    ui.horizontal(|ui| {
        ui.style_mut().spacing.item_spacing = [20.0, 0.0].into();

        for channel_type in mutual_channel_types {
            match channel_type {
                fixture::channel::FIXTURE_CHANNEL_INTENSITY_ID
                | fixture::channel::FIXTURE_CHANNEL_STROBE
                | fixture::channel::FIXTURE_CHANNEL_ZOOM => {
                    ui.vertical(|ui| {
                        ui.label(FixtureChannel::name_by_id(channel_type));
                        ui.add(eframe::egui::Slider::from_get_set(0.0..=1.0, |val| {
                            if let Some(val) = val {
                                for fixture_id in selected_fixtures.iter() {
                                    let intens = context
                                        .fixture_handler
                                        .fixture(*fixture_id)
                                        .unwrap()
                                        .channel_single_value_ref(channel_type)
                                        .expect("");
                                    *intens = val as f32;
                                }

                                val
                            } else {
                                context
                                    .fixture_handler
                                    .fixture(selected_fixtures[0])
                                    .unwrap()
                                    .channel_single_value(channel_type)
                                    .expect("") as f64
                            }
                        }));
                    });
                }
                fixture::channel::FIXTURE_CHANNEL_COLOR_ID => {
                    ui.vertical(|ui| {
                        ui.label("Color RGB");

                        /*let fixture_color = context
                            .fixture_handler
                            .fixture(selected_fixtures[0])
                            .unwrap()
                            .color_rgb()
                            .expect("");

                        let c = ui.color_edit_button_rgb(
                            context
                                .fixture_handler
                                .fixture(selected_fixtures[0])
                                .unwrap()
                                .color_rgb_ref()
                                .expect(""),
                        );

                        if c.changed() || c.clicked() {
                            for fixture_id in selected_fixtures.iter().skip(1) {
                                let color = context
                                    .fixture_handler
                                    .fixture(*fixture_id)
                                    .unwrap()
                                    .color_rgb_ref()
                                    .expect("");

                                color.clone_from(&fixture_color)
                            }
                            }*/
                    });
                }
                fixture::channel::FIXTURE_CHANNEL_POSITION_PAN_TILT_ID => {
                    ui.vertical(|ui| {
                        ui.label("Pan/Tilt");
                        /*ui.add(PositionSelector::new(|val| {
                        if let Some(val) = val {
                            for fixture_id in selected_fixtures.iter() {
                                let position = context
                                    .fixture_handler
                                    .fixture(*fixture_id)
                                    .unwrap()
                                    .position_pan_tilt_ref()
                                    .expect("");
                                *position = val.into();
                            }

                            Some(eframe::egui::vec2(0.0, 0.0))
                        } else {
                            let pos = context
                                .fixture_handler
                                .fixture(selected_fixtures[0])
                                .unwrap()
                                .position_pan_tilt()
                                .expect("");

                            Some(eframe::egui::vec2(pos[0], pos[1]))
                        }
                        }));*/
                    });
                }
                fixture::channel::FIXTURE_CHANNEL_TOGGLE_FLAGS => {
                    ui.style_mut().spacing.item_spacing = [0.0, 10.0].into();
                    ui.style_mut().wrap = Some(false);

                    ui.vertical(|ui| {
                        ui.label("Toggle flags");

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
                    ui.label("Not supported");
                }
            }

            ui.separator();
        }
    });
}
