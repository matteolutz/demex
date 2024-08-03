use crate::{lexer::token::Token, ui::DemexUiContext};

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut DemexUiContext) -> () {
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

        for group in context.preset_handler.groups().values() {
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

        for preset in context.preset_handler.colors().values() {
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
                Token::Numeral(*context.preset_handler.colors().keys().max().unwrap_or(&0) + 1),
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

        for preset in context.preset_handler.positions().values() {
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
                        .positions()
                        .keys()
                        .max()
                        .unwrap_or(&0)
                        + 1,
                ),
            ])
        }

        ui.end_row();
    });
}
