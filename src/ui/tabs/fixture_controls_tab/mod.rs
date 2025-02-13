use color::color_controls_ui;
use position::position_controls_ui;
use toggle_flags::toggle_flags_controls_ui;

use crate::{
    fixture::{
        self,
        channel::value::{FixtureChannelValue, FixtureChannelValueTrait},
    },
    parser::nodes::fixture_selector::FixtureSelectorContext,
    ui::components::slider::was_touched_slider,
};

pub mod color;
pub mod position;
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
        .unwrap()
        .get_fixtures(
            &preset_handler,
            FixtureSelectorContext::new(&context.global_fixture_select),
        )
        .expect("fixture selection failed");

    let mut mutual_channel_types = fixture_handler
        .fixture_immut(selected_fixtures[0])
        .unwrap()
        .channel_types()
        .clone();

    for fixture_id in selected_fixtures.iter().skip(1) {
        let fixture_channel_types = fixture_handler
            .fixture_immut(*fixture_id)
            .unwrap()
            .channel_types();

        mutual_channel_types.retain(|channel_type| fixture_channel_types.contains(channel_type));
    }

    ui.style_mut().spacing.item_spacing = [0.0, 20.0].into();

    ui.heading("Fixture Controls");

    ui.horizontal(|ui| {
        ui.style_mut().spacing.item_spacing = [20.0, 10.0].into();

        for channel_type in mutual_channel_types {
            let channel_name = fixture_handler
                .fixture(selected_fixtures[0])
                .unwrap()
                .channel_name(channel_type)
                .unwrap();

            let is_channel_home = fixture_handler
                .fixture(selected_fixtures[0])
                .unwrap()
                .channel_value_programmer(channel_type)
                .map(|v| v.is_home())
                .unwrap_or(false);

            ui.vertical(|ui| {
                match channel_type {
                    fixture::channel::FIXTURE_CHANNEL_INTENSITY_ID
                    | fixture::channel::FIXTURE_CHANNEL_SHUTTER_ID
                    | fixture::channel::FIXTURE_CHANNEL_ZOOM_ID => {
                        ui.vertical(|ui| {
                            ui.set_width(100.0);
                            ui.label(egui::RichText::from(channel_name).color(
                                if is_channel_home {
                                    egui::Color32::PLACEHOLDER
                                } else {
                                    egui::Color32::YELLOW
                                },
                            ));

                            let mut value = fixture_handler
                                .fixture_immut(selected_fixtures[0])
                                .unwrap()
                                .channel_single_value_programmer(channel_type, &preset_handler)
                                .unwrap();

                            if was_touched_slider(ui, &mut value) {
                                for fixture_id in selected_fixtures.iter() {
                                    fixture_handler
                                        .fixture(*fixture_id)
                                        .unwrap()
                                        .set_channel_single_value(channel_type, value)
                                        .expect("");
                                }
                            }
                        });
                    }
                    fixture::channel::FIXTURE_CHANNEL_COLOR_ID => {
                        ui.set_width(100.0);

                        color_controls_ui(
                            ui,
                            &channel_name,
                            is_channel_home,
                            &selected_fixtures,
                            &preset_handler,
                            &mut fixture_handler,
                        );
                    }
                    fixture::channel::FIXTURE_CHANNEL_POSITION_PAN_TILT_ID => {
                        position_controls_ui(
                            ui,
                            &channel_name,
                            is_channel_home,
                            &selected_fixtures,
                            &preset_handler,
                            &mut fixture_handler,
                        );
                    }
                    fixture::channel::FIXTURE_CHANNEL_TOGGLE_FLAGS => {
                        ui.style_mut().spacing.item_spacing = [0.0, 10.0].into();
                        ui.style_mut().wrap_mode = Some(eframe::egui::TextWrapMode::Extend);

                        toggle_flags_controls_ui(
                            ui,
                            &channel_name,
                            is_channel_home,
                            &selected_fixtures,
                            &mut fixture_handler,
                        );
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
                                            fixture_handler
                                                .fixture(*fixture_id)
                                                .unwrap()
                                                .set_channel_single_value(channel_type, val as f32)
                                                .expect("");
                                        }

                                        val
                                    } else {
                                        fixture_handler
                                            .fixture(selected_fixtures[0])
                                            .unwrap()
                                            .channel_single_value_programmer(
                                                channel_type,
                                                &preset_handler,
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
                        fixture_handler
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
