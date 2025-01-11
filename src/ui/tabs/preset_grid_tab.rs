use itertools::Itertools;

use crate::{
    fixture::channel::{FIXTURE_CHANNEL_COLOR_ID, FIXTURE_CHANNEL_POSITION_PAN_TILT_ID},
    lexer::token::Token,
    parser::nodes::fixture_selector::FixtureSelectorContext,
    ui::DemexUiContext,
};

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut DemexUiContext) {
    eframe::egui::Grid::new("preset_grid").show(ui, |ui| {
        // Groups
        ui.add_sized(
            [80.0, 80.0],
            eframe::egui::Button::new("Groups")
                .stroke(eframe::egui::Stroke::new(1.0, eframe::egui::Color32::RED))
                .sense(eframe::egui::Sense {
                    click: false,
                    drag: false,
                    focusable: false,
                }),
        );

        for (_, group) in context
            .preset_handler
            .groups()
            .iter()
            .sorted_by(|a, b| a.0.cmp(b.0))
        {
            let group_button = ui.add_sized([80.0, 80.0], eframe::egui::Button::new(group.name()));
            if group_button.clicked() {
                context
                    .command
                    .extend(vec![Token::KeywordGroup, Token::Numeral(group.id())])
            }
        }

        let add_group_button = ui.add_sized([80.0, 80.0], eframe::egui::Button::new("+"));
        if add_group_button.clicked() {
            context.command.extend(vec![
                Token::KeywordRecord,
                Token::KeywordGroup,
                Token::Numeral(*context.preset_handler.groups().keys().max().unwrap_or(&0) + 1),
            ])
        }

        ui.end_row();
        ui.end_row();

        // Colors
        ui.add_sized(
            [80.0, 80.0],
            eframe::egui::Button::new("Color")
                .stroke(eframe::egui::Stroke::new(1.0, eframe::egui::Color32::GREEN))
                .sense(eframe::egui::Sense {
                    click: false,
                    drag: false,
                    focusable: false,
                }),
        );

        for (_, preset) in context
            .preset_handler
            .presets(FIXTURE_CHANNEL_COLOR_ID)
            .iter()
            .sorted_by(|a, b| a.0.cmp(b.0))
        {
            let preset_button =
                ui.add_sized([80.0, 80.0], eframe::egui::Button::new(preset.name()));
            if preset_button.clicked() {
                context.command.extend(vec![
                    Token::KeywordColor,
                    Token::KeywordPreset,
                    Token::Numeral(preset.id()),
                ])
            }
        }

        let add_color_button = ui.add_sized([80.0, 80.0], eframe::egui::Button::new("+"));
        if add_color_button.clicked() {
            context.command.extend(vec![
                Token::KeywordRecord,
                Token::KeywordColor,
                Token::Numeral(
                    *context
                        .preset_handler
                        .presets(FIXTURE_CHANNEL_COLOR_ID)
                        .keys()
                        .max()
                        .unwrap_or(&0)
                        + 1,
                ),
            ])
        }

        ui.end_row();
        ui.end_row();

        // Positions
        ui.add_sized(
            [80.0, 80.0],
            eframe::egui::Button::new("Position")
                .stroke(eframe::egui::Stroke::new(1.0, eframe::egui::Color32::BLUE))
                .sense(eframe::egui::Sense {
                    click: false,
                    drag: false,
                    focusable: false,
                }),
        );

        for (_, preset) in context
            .preset_handler
            .presets(FIXTURE_CHANNEL_POSITION_PAN_TILT_ID)
            .iter()
            .sorted_by(|a, b| a.0.cmp(b.0))
        {
            let preset_button =
                ui.add_sized([80.0, 80.0], eframe::egui::Button::new(preset.name()));
            if preset_button.clicked() {
                context.command.extend(vec![
                    Token::KeywordPosition,
                    Token::KeywordPreset,
                    Token::Numeral(preset.id()),
                ])
            }
        }

        let add_position_button = ui.add_sized([80.0, 80.0], eframe::egui::Button::new("+"));
        if add_position_button.clicked() {
            context.command.extend(vec![
                Token::KeywordRecord,
                Token::KeywordPosition,
                Token::Numeral(
                    *context
                        .preset_handler
                        .presets(FIXTURE_CHANNEL_POSITION_PAN_TILT_ID)
                        .keys()
                        .max()
                        .unwrap_or(&0)
                        + 1,
                ),
            ])
        }

        ui.end_row();
        ui.end_row();

        // Macros
        ui.add_sized(
            [80.0, 80.0],
            eframe::egui::Button::new("Macros")
                .stroke(eframe::egui::Stroke::new(
                    1.0,
                    eframe::egui::Color32::YELLOW,
                ))
                .sense(eframe::egui::Sense {
                    click: false,
                    drag: false,
                    focusable: false,
                }),
        );

        for (_, preset) in context
            .preset_handler
            .macros()
            .clone() // i hate myself for doing this
            .iter()
            .sorted_by(|a, b| a.0.cmp(b.0))
        {
            let preset_button =
                ui.add_sized([80.0, 80.0], eframe::egui::Button::new(preset.name()));
            if preset_button.clicked() {
                let macro_run_result = preset.run(
                    &mut context.fixture_handler,
                    &mut context.preset_handler,
                    FixtureSelectorContext::new(&context.global_fixture_select),
                );
                println!("Macro run result: {:?}", macro_run_result);
            }
        }

        let add_position_button = ui.add_sized([80.0, 80.0], eframe::egui::Button::new("+"));
        if add_position_button.clicked() {
            context.command.extend(vec![
                Token::KeywordRecord,
                Token::KeywordMacro,
                Token::Numeral(*context.preset_handler.macros().keys().max().unwrap_or(&0) + 1),
            ])
        }

        ui.end_row();
        ui.end_row();

        // Command Slices
        ui.add_sized(
            [80.0, 80.0],
            eframe::egui::Button::new("Command Slices")
                .stroke(eframe::egui::Stroke::new(
                    1.0,
                    eframe::egui::Color32::LIGHT_YELLOW,
                ))
                .sense(eframe::egui::Sense {
                    click: false,
                    drag: false,
                    focusable: false,
                }),
        );

        for (_, preset) in context
            .preset_handler
            .command_slices()
            .iter()
            .sorted_by(|a, b| a.0.cmp(b.0))
        {
            let preset_button =
                ui.add_sized([80.0, 80.0], eframe::egui::Button::new(preset.name()));
            if preset_button.clicked() {
                context.command.extend(preset.command().clone());
            }
        }

        let add_position_button = ui.add_sized([80.0, 80.0], eframe::egui::Button::new("+"));
        if add_position_button.clicked() {
            // TODO: add command slices command
        }

        ui.end_row();
    });
}
