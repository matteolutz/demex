use button::{
    preset_grid_button_ui, PresetGridButton, PresetGridButtonConfig, PresetGridButtonDecoration,
    PresetGridButtonQuickMenuActions,
};
use row::preset_grid_row_ui;
use strum::IntoEnumIterator;

use crate::{
    fixture::{
        channel3::feature::feature_group::FixtureChannel3FeatureGroup,
        presets::preset::{FixturePresetData, FixturePresetId},
        updatables::executor::config::ExecutorConfig,
    },
    lexer::token::Token,
    ui::{
        window::{edit::DemexEditWindow, DemexWindow},
        DemexUiContext,
    },
};

pub const PRESET_GRID_ELEMENT_SIZE: [f32; 2] = [80.0, 80.0];

mod button;
mod row;

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut DemexUiContext) {
    let mut fixture_handler = context.fixture_handler.write();
    let preset_handler = context.preset_handler.read();
    let mut updatable_handler = context.updatable_handler.write();
    let patch = context.patch.read();

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
                        display_color: None,
                    }
                } else {
                    PresetGridButtonConfig::Empty { id }
                };

                let (response, quick_action) = PresetGridButton::new(
                    config,
                    PresetGridButtonDecoration::default(),
                    None,
                    None,
                )
                .show(ui);

                if response.clicked()
                    || quick_action.is_some_and(|a| a == PresetGridButtonQuickMenuActions::Default)
                {
                    if let Ok(g) = g {
                        context.global_fixture_select = Some(g.fixture_selection().clone());
                    }
                }

                if let Some(quick_action) = quick_action {
                    match quick_action {
                        PresetGridButtonQuickMenuActions::Insert => {
                            context
                                .command
                                .extend_from_slice(&[Token::KeywordGroup, Token::Integer(id)]);
                        }

                        PresetGridButtonQuickMenuActions::New => {
                            context.command.extend_from_slice(&[
                                Token::KeywordRecord,
                                Token::KeywordGroup,
                                Token::Integer(id),
                            ]);
                        }
                        PresetGridButtonQuickMenuActions::Edit => {
                            if let Ok(g) = g {
                                context.window_handler.add_window(DemexWindow::Edit(
                                    DemexEditWindow::EditGroup(g.id()),
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }
        });

        // Feature Presets
        for feature_group in FixtureChannel3FeatureGroup::iter() {
            preset_grid_row_ui(
                ui,
                feature_group.name(),
                Some(feature_group.into()),
                egui::Color32::BLUE,
                |ui| {
                    for id in 0..=preset_handler.next_preset_id(feature_group) {
                        let preset_id = FixturePresetId {
                            feature_group,
                            preset_id: id,
                        };

                        let p = preset_handler.get_preset(preset_id);

                        let config = if let Ok(p) = p {
                            PresetGridButtonConfig::Preset {
                                id: p.id().preset_id,
                                name: p.name().to_owned(),
                                top_bar_color: selected_fixtures
                                    .as_ref()
                                    .map(|selected| p.get_target(selected))
                                    .map(|target| target.get_color()),
                                display_color: p.display_color(),
                            }
                        } else {
                            PresetGridButtonConfig::Empty { id }
                        };

                        let (response, quick_action) = PresetGridButton::new(
                            config,
                            PresetGridButtonDecoration {
                                left_bottom_text: None,
                                right_top_text: p.as_ref().ok().and_then(|p| match p.data() {
                                    FixturePresetData::FeatureEffect { .. } => {
                                        Some("FeFX".to_owned())
                                    }
                                    _ => None,
                                }),
                            },
                            None,
                            Some(vec![PresetGridButtonQuickMenuActions::Custom(
                                "New (Create)",
                            )]),
                        )
                        .show(ui);

                        if let Some(quick_action) = quick_action {
                            match quick_action {
                                PresetGridButtonQuickMenuActions::Insert => {
                                    context.command.extend_from_slice(&[
                                        Token::KeywordPreset,
                                        Token::FloatingPoint(0.0, (feature_group.into(), id)),
                                    ]);
                                }
                                PresetGridButtonQuickMenuActions::New => {
                                    context.command.extend_from_slice(&[
                                        Token::KeywordRecord,
                                        Token::KeywordPreset,
                                        Token::FloatingPoint(0.0, (feature_group.into(), id)),
                                    ]);
                                }
                                PresetGridButtonQuickMenuActions::Custom("New (Create)") => {
                                    context.command.extend_from_slice(&[
                                        Token::KeywordCreate,
                                        Token::KeywordPreset,
                                        Token::FloatingPoint(0.0, (feature_group.into(), id)),
                                    ]);
                                }
                                PresetGridButtonQuickMenuActions::Edit => {
                                    if let Ok(p) = p {
                                        context.window_handler.add_window(DemexWindow::Edit(
                                            DemexEditWindow::EditPreset(p.id()),
                                        ));
                                    }
                                }
                                _ => {}
                            }
                        }

                        if response.clicked()
                            || quick_action
                                .is_some_and(|a| a == PresetGridButtonQuickMenuActions::Default)
                        {
                            if let (Ok(_), Some(selection)) =
                                (p.as_ref(), context.global_fixture_select.as_ref())
                            {
                                preset_handler
                                    .apply_preset(
                                        preset_id,
                                        &mut fixture_handler,
                                        patch.fixture_types(),
                                        selection.clone(),
                                    )
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
                        display_color: None,
                    }
                } else {
                    PresetGridButtonConfig::Empty { id }
                };

                /*let response =
                preset_grid_button_ui(ui, config, PresetGridButtonDecoration::default());*/
                let (response, quick_action) = PresetGridButton::new(
                    config,
                    PresetGridButtonDecoration::default(),
                    None,
                    None,
                )
                .show(ui);

                if response.clicked()
                    || quick_action
                        .is_some_and(|action| action == PresetGridButtonQuickMenuActions::Default)
                {
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

                if let Some(quick_action) = quick_action {
                    match quick_action {
                        PresetGridButtonQuickMenuActions::Insert => {
                            context
                                .command
                                .extend_from_slice(&[Token::KeywordMacro, Token::Integer(id)]);
                        }

                        PresetGridButtonQuickMenuActions::New => {
                            context.command.extend_from_slice(&[
                                Token::KeywordCreate,
                                Token::KeywordMacro,
                                Token::Integer(id),
                            ]);
                        }
                        PresetGridButtonQuickMenuActions::Edit => {
                            todo!()
                        }
                        _ => {}
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
                        display_color: None,
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
                        display_color: None,
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

                // let response = preset_grid_button_ui(ui, config, decoration);
                let (response, quick_action) = PresetGridButton::new(
                    config,
                    decoration,
                    Some(vec![PresetGridButtonQuickMenuActions::Custom("Stop")]),
                    None,
                )
                .show(ui);

                if response.clicked()
                    || quick_action.is_some_and(|a| a == PresetGridButtonQuickMenuActions::Default)
                {
                    if executor_exists {
                        updatable_handler
                            .start_or_next_executor(id, &mut fixture_handler, &preset_handler, 0.0)
                            .unwrap();
                    } else {
                        context.command.extend_from_slice(&[
                            Token::KeywordCreate,
                            Token::KeywordExecutor,
                            Token::Integer(id),
                        ]);
                    }
                }

                if let Some(quick_action) = quick_action {
                    match quick_action {
                        PresetGridButtonQuickMenuActions::Insert => {
                            context
                                .command
                                .extend_from_slice(&[Token::KeywordExecutor, Token::Integer(id)]);
                        }

                        PresetGridButtonQuickMenuActions::New => {
                            context.command.extend_from_slice(&[
                                Token::KeywordCreate,
                                Token::KeywordExecutor,
                                Token::Integer(id),
                            ]);
                        }
                        PresetGridButtonQuickMenuActions::Edit => {
                            if executor_exists {
                                context.window_handler.add_window(DemexWindow::Edit(
                                    DemexEditWindow::EditExecutor(id),
                                ));
                            }
                        }
                        _ => {}
                    }
                }

                if (response.secondary_clicked()
                    || quick_action
                        .is_some_and(|a| a == PresetGridButtonQuickMenuActions::Custom("Stop")))
                    && executor_exists
                    && updatable_handler.executor(id).unwrap().is_started()
                {
                    updatable_handler
                        .stop_executor(id, &mut fixture_handler, &preset_handler)
                        .unwrap();
                }
            }
        });
    });
}
