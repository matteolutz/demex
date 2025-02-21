use button::{preset_grid_button_ui, PresetGridButtonConfig, PresetGridButtonDecoration};
use itertools::Itertools;
use row::preset_grid_row_ui;

use crate::{
    fixture::{presets::preset::FixturePresetId, updatables::executor::config::ExecutorConfig},
    lexer::token::Token,
    ui::DemexUiContext,
};

pub const PRESET_GRID_ELEMENT_SIZE: [f32; 2] = [80.0, 80.0];

mod button;
mod row;

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut DemexUiContext) {
    let mut fixture_handler = context.fixture_handler.write();
    let preset_handler = context.preset_handler.read();
    let mut updatable_handler = context.updatable_handler.write();

    let _selected_fixtures = context
        .global_fixture_select
        .as_ref()
        .map(|selection| selection.fixtures().to_vec());
    let selected_fixtures = _selected_fixtures.as_ref();

    ui.vertical(|ui| {
        // Groups
        preset_grid_row_ui(ui, "Groups", None, egui::Color32::DARK_RED, |ui| {
            for id in 0..=preset_handler.next_group_id() {
                let g = preset_handler.get_group(id);

                let config = if let Ok(g) = g {
                    PresetGridButtonConfig::Preset {
                        id: g.id(),
                        name: g.name().to_owned(),
                        top_bar_color: context.global_fixture_select.as_ref().and_then(
                            |selection| {
                                if selection == g.fixture_selection() {
                                    Some(egui::Color32::GREEN)
                                } else {
                                    None
                                }
                            },
                        ),
                    }
                } else {
                    PresetGridButtonConfig::Empty { id }
                };

                let response =
                    preset_grid_button_ui(ui, config, PresetGridButtonDecoration::default());

                if response.clicked() {
                    if g.is_ok() {
                        context
                            .command
                            .extend_from_slice(&[Token::KeywordGroup, Token::Integer(id)]);
                    } else {
                        context.command.extend_from_slice(&[
                            Token::KeywordRecord,
                            Token::KeywordGroup,
                            Token::Integer(id),
                        ]);
                    }
                }

                if response.double_clicked() {
                    context.command.clear();
                    context.command_input.clear();
                    context.is_command_input_empty = true;

                    if let Ok(g) = g {
                        context.global_fixture_select = Some(g.fixture_selection().clone());
                    }
                }
            }
        });

        // Feature Presets
        for (feature_group_id, feature_group) in preset_handler
            .feature_groups()
            .iter()
            .sorted_by_key(|(id, _)| *id)
        {
            preset_grid_row_ui(
                ui,
                feature_group.name(),
                Some(*feature_group_id),
                egui::Color32::BLUE,
                |ui| {
                    for id in 0..=preset_handler.next_preset_id(*feature_group_id) {
                        let p = preset_handler.get_preset(FixturePresetId {
                            feature_group_id: *feature_group_id,
                            preset_id: id,
                        });

                        let config = if let Ok(p) = p {
                            PresetGridButtonConfig::Preset {
                                id: p.id().preset_id,
                                name: p.name().to_owned(),
                                top_bar_color: selected_fixtures
                                    .as_ref()
                                    .map(|selected| p.get_target(selected))
                                    .map(|target| target.get_color()),
                            }
                        } else {
                            PresetGridButtonConfig::Empty { id }
                        };

                        let response = preset_grid_button_ui(
                            ui,
                            config,
                            PresetGridButtonDecoration::default(),
                        );

                        if response.clicked() {
                            if p.is_ok() {
                                context.command.extend_from_slice(&[
                                    Token::KeywordPreset,
                                    Token::FloatingPoint(0.0, (*feature_group_id, id)),
                                ]);
                            } else {
                                context.command.extend_from_slice(&[
                                    Token::KeywordRecord,
                                    Token::KeywordPreset,
                                    Token::FloatingPoint(0.0, (*feature_group_id, id)),
                                ]);
                            }
                        }

                        if p.is_ok() && response.double_clicked() && selected_fixtures.is_some() {
                            context.command.clear();
                            context.command_input.clear();
                            context.is_command_input_empty = true;

                            for fixture_id in selected_fixtures.unwrap() {
                                p.as_ref()
                                    .unwrap()
                                    .apply(fixture_handler.fixture(*fixture_id).unwrap())
                                    .unwrap();
                            }
                        }
                    }
                },
            );
        }

        // Macros
        preset_grid_row_ui(ui, "Maros", None, egui::Color32::BROWN, |ui| {
            for id in 0..=preset_handler.next_macro_id() {
                let m = preset_handler.get_macro(id);

                let config = if let Ok(m) = m {
                    PresetGridButtonConfig::Preset {
                        id: m.id(),
                        name: m.name().to_owned(),
                        top_bar_color: None,
                    }
                } else {
                    PresetGridButtonConfig::Empty { id }
                };

                let response =
                    preset_grid_button_ui(ui, config, PresetGridButtonDecoration::default());

                if response.clicked() {
                    if let Ok(m) = m {
                        context.macro_execution_queue.push(m.action().clone());
                    } else {
                        context.command.extend_from_slice(&[
                            Token::KeywordCreate,
                            Token::KeywordMacro,
                            Token::Integer(id),
                        ]);
                    }
                }
            }
        });

        // Command slices
        preset_grid_row_ui(ui, "Command Slices", None, egui::Color32::GOLD, |ui| {
            for id in 0..=preset_handler.next_command_slice_id() {
                let cs = preset_handler.get_command_slice(id);

                let config = if let Ok(cs) = cs {
                    PresetGridButtonConfig::Preset {
                        id: cs.id(),
                        name: cs.name().to_owned(),
                        top_bar_color: None,
                    }
                } else {
                    PresetGridButtonConfig::Empty { id }
                };

                let response =
                    preset_grid_button_ui(ui, config, PresetGridButtonDecoration::default());

                if response.clicked() {
                    if let Ok(cs) = cs {
                        context.command.extend_from_slice(cs.command());
                    }
                }
            }
        });

        // Executors
        preset_grid_row_ui(ui, "Executors", None, egui::Color32::DARK_GREEN, |ui| {
            for id in 0..=updatable_handler.next_executor_id() {
                let executor_exists = updatable_handler.executor(id).is_some();

                let (config, decoration) = if executor_exists {
                    let executor = updatable_handler.executor(id).unwrap();

                    let config = PresetGridButtonConfig::Preset {
                        id,
                        name: executor.name().to_owned(),
                        top_bar_color: if updatable_handler.executor(id).unwrap().is_started() {
                            Some(egui::Color32::RED)
                        } else {
                            None
                        },
                    };

                    let decoration = match executor.config() {
                        ExecutorConfig::Sequence { runtime, .. } => PresetGridButtonDecoration {
                            right_top_text: Some(format!(
                                "{}/{}",
                                runtime
                                    .current_cue()
                                    .map(|c| (c + 1).to_string())
                                    .unwrap_or("-".to_owned()),
                                runtime.num_cues(&preset_handler)
                            )),
                            left_bottom_text: Some("Seq".to_owned()),
                        },
                        ExecutorConfig::FeatureEffect { .. } => PresetGridButtonDecoration {
                            right_top_text: None,
                            left_bottom_text: Some("FeFX".to_owned()),
                        },
                    };

                    (config, decoration)
                } else {
                    (
                        PresetGridButtonConfig::Empty { id },
                        PresetGridButtonDecoration::default(),
                    )
                };

                let response = preset_grid_button_ui(ui, config, decoration);

                if response.clicked() {
                    if executor_exists {
                        updatable_handler
                            .start_or_next_executor(id, &mut fixture_handler, &preset_handler)
                            .unwrap();
                    } else {
                        context.command.extend_from_slice(&[
                            Token::KeywordCreate,
                            Token::KeywordExecutor,
                            Token::Integer(id),
                        ]);
                    }
                }

                if response.secondary_clicked() && executor_exists {
                    if updatable_handler.executor(id).unwrap().is_started() {
                        updatable_handler
                            .stop_executor(id, &mut fixture_handler)
                            .unwrap();
                    } else {
                        context
                            .command
                            .extend_from_slice(&[Token::KeywordExecutor, Token::Integer(id)]);
                    }
                }
            }
        });

        /*
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
                                click: false, drag: false, focusable: false,
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
                            context.command.extend_from_slice(&[
                                Token::KeywordPreset,
                                Token::Integer(preset.id()),
                            ])
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
                        if updatable_handler.executor(*preset_id).unwrap().is_started() {
                            updatable_handler
                                .stop_executor(*preset_id, &mut fixture_handler)
                                .unwrap();
                        } else {
                            context.command.extend_from_slice(&[
                                Token::KeywordExecutor,
                                Token::Integer(*preset_id),
                            ])
                        }
                    }
                }

                let add_executor_button =
                    ui.add_sized([80.0, 80.0], eframe::egui::Button::new("+"));
                if add_executor_button.clicked() {
                    context.command.extend_from_slice(&[
                        Token::KeywordCreate,
                        Token::KeywordExecutor,
                        Token::KeywordNext,
                    ])
                }

                ui.end_row();
            });*/
    });
}
