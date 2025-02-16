use itertools::Itertools;

use crate::{
    lexer::token::Token,
    parser::nodes::fixture_selector::{
        AtomicFixtureSelector, FixtureSelector, FixtureSelectorContext,
    },
    ui::DemexUiContext,
};

const NUM_KEYS: [egui::Key; 10] = [
    egui::Key::Num1,
    egui::Key::Num2,
    egui::Key::Num3,
    egui::Key::Num4,
    egui::Key::Num5,
    egui::Key::Num6,
    egui::Key::Num7,
    egui::Key::Num8,
    egui::Key::Num9,
    egui::Key::Num0,
];

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut DemexUiContext) {
    let mut fixture_handler = context.fixture_handler.write();
    let preset_handler = context.preset_handler.read();
    let mut updatable_handler = context.updatable_handler.write();

    let selected_fixtures = context.global_fixture_select.as_ref().and_then(|fs| {
        fs.get_fixtures(
            &preset_handler,
            FixtureSelectorContext::new(&context.global_fixture_select),
        )
        .ok()
    });

    eframe::egui::Grid::new("preset_grid")
        .spacing((5.0, 5.0))
        .show(ui, |ui| {
            // Groups
            ui.add_sized(
                [80.0, 80.0],
                eframe::egui::Button::new("Groups")
                    .stroke(eframe::egui::Stroke::new(1.0, eframe::egui::Color32::RED))
                    .wrap()
                    .sense(eframe::egui::Sense {
                        click: false,
                        drag: false,
                        focusable: false,
                    }),
            );

            for (_, group) in preset_handler
                .groups()
                .iter()
                .sorted_by(|a, b| a.0.cmp(b.0))
            {
                let group_button =
                    ui.add_sized([80.0, 80.0], eframe::egui::Button::new(group.name()).wrap());
                if group_button.clicked() {
                    context
                        .command
                        .extend_from_slice(&[Token::KeywordGroup, Token::Integer(group.id())])
                }

                if group_button.double_clicked() {
                    context.command.clear();
                    context.command_input.clear();
                    context.is_command_input_empty = true;

                    context.global_fixture_select = Some(FixtureSelector::Atomic(
                        AtomicFixtureSelector::FixtureGroup(group.id()),
                    ))
                }
            }

            let add_group_button = ui.add_sized([80.0, 80.0], eframe::egui::Button::new("+"));
            if add_group_button.clicked() {
                context.command.extend_from_slice(&[
                    Token::KeywordRecord,
                    Token::KeywordGroup,
                    Token::Integer(preset_handler.groups().keys().max().unwrap_or(&0) + 1),
                ])
            }

            ui.end_row();

            // Feature group presets
            for (feature_group_id, feature_group) in preset_handler
                .feature_groups()
                .iter()
                .sorted_by_key(|(id, _)| *id)
            {
                ui.add_sized(
                    [80.0, 80.0],
                    eframe::egui::Button::new(feature_group.name())
                        .stroke(eframe::egui::Stroke::new(1.0, eframe::egui::Color32::GREEN))
                        .wrap()
                        .sense(eframe::egui::Sense {
                            click: false,
                            drag: false,
                            focusable: false,
                        }),
                );

                for preset in preset_handler
                    .presets_for_feature_group(*feature_group_id)
                    .iter()
                    .sorted_by_key(|p| p.id())
                {
                    let mut preset_button_text = egui::text::LayoutJob::single_section(
                        preset.name().to_owned(),
                        egui::TextFormat::simple(
                            egui::FontId::proportional(12.0),
                            egui::Color32::PLACEHOLDER,
                        ),
                    );

                    if let Some(selected_fixtures) = &selected_fixtures {
                        let target_mode = preset.get_target(selected_fixtures);

                        preset_button_text.append(
                            format!("\n{}", target_mode.get_short_name()).as_str(),
                            0.0,
                            egui::TextFormat::simple(
                                egui::FontId::monospace(8.0),
                                target_mode.get_color(),
                            ),
                        );
                    }

                    let preset_button = ui.add_sized(
                        [80.0, 80.0],
                        eframe::egui::Button::new(preset_button_text).wrap(),
                    );

                    if preset_button.clicked() {
                        context
                            .command
                            .extend_from_slice(&[Token::KeywordPreset, Token::Integer(preset.id())])
                    }

                    if preset_button.double_clicked() && selected_fixtures.is_some() {
                        context.command.clear();
                        context.command_input.clear();
                        context.is_command_input_empty = true;

                        for fixture_id in selected_fixtures.as_ref().unwrap() {
                            preset
                                .apply(fixture_handler.fixture(*fixture_id).unwrap())
                                .unwrap();
                        }
                    }
                }

                let record_preset_button =
                    ui.add_sized([80.0, 80.0], eframe::egui::Button::new("+"));
                if record_preset_button.clicked() {
                    context.command.extend_from_slice(&[
                        Token::KeywordRecord,
                        Token::KeywordPreset,
                        Token::KeywordFeature,
                        Token::Integer(*feature_group_id),
                        Token::KeywordNext,
                    ]);
                }

                ui.end_row();
            }

            // Macros
            ui.add_sized(
                [80.0, 80.0],
                eframe::egui::Button::new("Macros")
                    .stroke(eframe::egui::Stroke::new(
                        1.0,
                        eframe::egui::Color32::YELLOW,
                    ))
                    .wrap()
                    .sense(eframe::egui::Sense {
                        click: false,
                        drag: false,
                        focusable: false,
                    }),
            );

            for (_, preset) in preset_handler
                .macros()
                .clone() // i hate myself for doing this
                .iter()
                .sorted_by(|a, b| a.0.cmp(b.0))
            {
                let preset_button = ui.add_sized(
                    [80.0, 80.0],
                    eframe::egui::Button::new(preset.name()).wrap(),
                );
                if preset_button.clicked() {
                    context.macro_execution_queue.push(preset.action().clone());
                }
            }

            let add_macro_button = ui.add_sized([80.0, 80.0], eframe::egui::Button::new("+"));
            if add_macro_button.clicked() {
                context.command.extend_from_slice(&[
                    Token::KeywordCreate,
                    Token::KeywordMacro,
                    Token::KeywordNext,
                ])
            }

            ui.end_row();

            // Command Slices
            ui.add_sized(
                [80.0, 80.0],
                eframe::egui::Button::new("Command Slices")
                    .stroke(eframe::egui::Stroke::new(
                        1.0,
                        eframe::egui::Color32::LIGHT_YELLOW,
                    ))
                    .wrap()
                    .sense(eframe::egui::Sense {
                        click: false,
                        drag: false,
                        focusable: false,
                    }),
            );

            for (_, preset) in preset_handler
                .command_slices()
                .iter()
                .sorted_by(|a, b| a.0.cmp(b.0))
            {
                let preset_button = ui.add_sized(
                    [80.0, 80.0],
                    eframe::egui::Button::new(preset.name()).wrap(),
                );
                if preset_button.clicked() {
                    context.command.extend_from_slice(preset.command());
                }
            }

            let add_command_slice_button =
                ui.add_sized([80.0, 80.0], eframe::egui::Button::new("+"));
            if add_command_slice_button.clicked() {
                // TODO: add command slices command
            }

            ui.end_row();

            // Executors
            ui.add_sized(
                [80.0, 80.0],
                eframe::egui::Button::new("Executors")
                    .stroke(eframe::egui::Stroke::new(
                        1.0,
                        eframe::egui::Color32::LIGHT_GREEN,
                    ))
                    .wrap()
                    .sense(eframe::egui::Sense {
                        click: false,
                        drag: false,
                        focusable: false,
                    }),
            );

            for (preset_idx, preset_id) in updatable_handler
                .executor_keys()
                .iter()
                .sorted_by(|a, b| a.cmp(b))
                .enumerate()
            {
                let is_started = updatable_handler.executor(*preset_id).unwrap().is_started();

                let preset_button = ui.add_sized(
                    [80.0, 80.0],
                    eframe::egui::Button::new(
                        updatable_handler
                            .executor(*preset_id)
                            .unwrap()
                            .to_string(&preset_handler),
                    )
                    .wrap()
                    .stroke(if is_started {
                        eframe::egui::Stroke::new(1.0, eframe::egui::Color32::RED)
                    } else {
                        eframe::egui::Stroke::NONE
                    }),
                );

                if preset_button.clicked()
                    || (preset_idx < 10
                        && ui.input(|reader| {
                            (reader.modifiers.ctrl || reader.modifiers.mac_cmd)
                                && reader.key_pressed(NUM_KEYS[preset_idx])
                        }))
                {
                    if is_started {
                        updatable_handler
                            .executor_mut(*preset_id)
                            .unwrap()
                            .next_cue(&preset_handler);
                    } else {
                        updatable_handler
                            .start_executor(*preset_id, &mut fixture_handler)
                            .unwrap();
                    }
                }

                if preset_button.secondary_clicked() {
                    updatable_handler
                        .stop_executor(*preset_id, &mut fixture_handler)
                        .unwrap();
                }

                if preset_button.long_touched() {
                    context
                        .command
                        .extend_from_slice(&[Token::KeywordExecutor, Token::Integer(*preset_id)])
                }
            }

            let add_executor_button = ui.add_sized([80.0, 80.0], eframe::egui::Button::new("+"));
            if add_executor_button.clicked() {
                context.command.extend_from_slice(&[
                    Token::KeywordCreate,
                    Token::KeywordExecutor,
                    Token::KeywordNext,
                ])
            }

            ui.end_row();
        });
}
