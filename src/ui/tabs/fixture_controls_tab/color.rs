use egui::color_picker::color_picker_color32;
use itertools::Itertools;

use crate::fixture::{
    channel::{
        value::{FixtureChannelDiscreteValue, FixtureChannelValue, FixtureChannelValueTrait},
        FixtureColorChannelMode, FIXTURE_CHANNEL_COLOR_ID,
    },
    handler::FixtureHandler,
    presets::PresetHandler,
};

const FIXTURES_DONT_SHARE_COLOR_MODE: &str = "The selected fixtures have different color modes. Please select them individually to change their color values.";

pub fn color_controls_ui(
    ui: &mut eframe::egui::Ui,
    channel_name: &str,
    is_channel_home: bool,
    selected_fixtures: &[u32],
    preset_handler: &PresetHandler,
    fixture_handler: &mut FixtureHandler,
) {
    ui.vertical(|ui| {
        ui.label(
            egui::RichText::from(channel_name).color(if is_channel_home {
                egui::Color32::PLACEHOLDER
            } else {
                egui::Color32::YELLOW
            }),
        );

        let color_modes = selected_fixtures
            .iter()
            .map(|f_id| {
                fixture_handler
                    .fixture(*f_id)
                    .unwrap()
                    .channel(FIXTURE_CHANNEL_COLOR_ID)
                    .unwrap()
                    .color_mode()
                    .unwrap()
            })
            .unique()
            .collect::<Vec<_>>();

        if color_modes.len() > 1 {
            ui.colored_label(egui::Color32::YELLOW, FIXTURES_DONT_SHARE_COLOR_MODE);
        } else {
            let fixture_color_channel_mode = fixture_handler
                .fixture(selected_fixtures[0])
                .unwrap()
                .channel(FIXTURE_CHANNEL_COLOR_ID)
                .unwrap()
                .color_mode()
                .unwrap();

            let fixture_color = fixture_handler
                .fixture(selected_fixtures[0])
                .unwrap()
                .color_programmer()
                .expect("");

            match fixture_color_channel_mode {
                FixtureColorChannelMode::Rgbw => {
                    let rgb_color = fixture_color
                        // TODO: change this, to use the corresponding fixture
                        .as_quadruple(
                            &preset_handler,
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
                    if color_picker_color32(ui, &mut color, egui::color_picker::Alpha::Opaque) {
                        for fixture_id in selected_fixtures.iter() {
                            fixture_handler
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
                }
                FixtureColorChannelMode::Macro => {
                    for (byte_value, color) in fixture_handler
                        .fixture(selected_fixtures[0])
                        .unwrap()
                        .channel(FIXTURE_CHANNEL_COLOR_ID)
                        .unwrap()
                        .color_macro_map()
                        .unwrap()
                        .clone()
                        .iter()
                        .sorted_by_key(|(byte_value, _)| *byte_value)
                    {
                        ui.scope(|ui| {
                            let egui_color = egui::Color32::from_rgb(
                                (color[0] * 255.0) as u8,
                                (color[1] * 255.0) as u8,
                                (color[2] * 255.0) as u8,
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
                                        .set_color(FixtureChannelValue::Discrete(
                                            FixtureChannelDiscreteValue::Single(
                                                *byte_value as f32 / 255.0,
                                            ),
                                        ))
                                        .expect("");
                                }
                            }
                        });
                    }
                }
            }
        }
    });
}
