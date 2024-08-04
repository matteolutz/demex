use egui::color_picker::color_picker_color32;

use crate::{
    fixture::{
        self,
        channel::{color::FixtureColorValue, position::FixturePositionValue, FixtureChannel},
    },
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

                        let fixture_color = context
                            .fixture_handler
                            .fixture(selected_fixtures[0])
                            .unwrap()
                            .color()
                            .expect("");

                        let rgb_color = match fixture_color {
                            FixtureColorValue::Rgbw(rgbw) => [rgbw[0], rgbw[1], rgbw[2]],
                            FixtureColorValue::Preset(preset_id) => context
                                .preset_handler
                                .get_color_for_fixture(preset_id, selected_fixtures[0])
                                .map(|color| [color[0], color[1], color[2]])
                                .unwrap_or([0.0, 0.0, 0.0]),
                        };

                        let mut color = eframe::egui::Color32::from_rgb(
                            (rgb_color[0] * 255.0) as u8,
                            (rgb_color[1] * 255.0) as u8,
                            (rgb_color[2] * 255.0) as u8,
                        );

                        if color_picker_color32(ui, &mut color, egui::color_picker::Alpha::Opaque) {
                            for fixture_id in selected_fixtures.iter() {
                                let f_color = context
                                    .fixture_handler
                                    .fixture(*fixture_id)
                                    .unwrap()
                                    .color_ref()
                                    .expect("");

                                *f_color = FixtureColorValue::from_rgb([
                                    color.r() as f32 / 255.0,
                                    color.g() as f32 / 255.0,
                                    color.b() as f32 / 255.0,
                                ]);
                            }
                        }
                    });
                }
                fixture::channel::FIXTURE_CHANNEL_POSITION_PAN_TILT_ID => {
                    ui.vertical(|ui| {
                        ui.label("Pan/Tilt");
                        ui.add(PositionSelector::new(|val| {
                            if let Some(val) = val {
                                for fixture_id in selected_fixtures.iter() {
                                    let position = context
                                        .fixture_handler
                                        .fixture(*fixture_id)
                                        .unwrap()
                                        .position_pan_tilt_ref()
                                        .expect("");
                                    *position = FixturePositionValue::PanTilt(val.into());
                                }

                                Some(eframe::egui::vec2(0.0, 0.0))
                            } else {
                                let pos_val = context
                                    .fixture_handler
                                    .fixture(selected_fixtures[0])
                                    .unwrap()
                                    .position_pan_tilt()
                                    .expect("");

                                let pos = match pos_val {
                                    FixturePositionValue::PanTilt(pos) => pos,
                                    FixturePositionValue::Preset(preset_id) => context
                                        .preset_handler
                                        .get_position_for_fixture(preset_id, selected_fixtures[0])
                                        .unwrap_or([0.0, 0.0]),
                                };

                                Some(eframe::egui::vec2(pos[0], pos[1]))
                            }
                        }));
                    });
                }
                fixture::channel::FIXTURE_CHANNEL_TOGGLE_FLAGS => {
                    ui.style_mut().spacing.item_spacing = [0.0, 10.0].into();
                    ui.style_mut().wrap_mode = Some(eframe::egui::TextWrapMode::Extend);

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
