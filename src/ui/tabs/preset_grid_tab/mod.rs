use button::{
    PresetGridButton, PresetGridButtonConfig, PresetGridButtonDecoration,
    PresetGridButtonQuickMenuActions,
};
use row::preset_grid_row_ui;
use strum::IntoEnumIterator;

use crate::{
    fixture::{
        channel3::feature::feature_group::FixtureChannel3FeatureGroup,
        presets::preset::{FixturePresetData, FixturePresetId},
    },
    lexer::token::Token,
    parser::nodes::action::{
        functions::{
            go_function::ExecutorGoArgs,
            set_function::{SelectionOrSelector, SetFixturePresetArgs},
            stop_function::ExecutorStopArgs,
        },
        Action, ValueOrRange,
    },
    ui::{
        edit_request::UiEditRequest,
        window::{edit::DemexEditWindow, DemexWindow},
        DemexUiContext,
    },
};

pub const PRESET_GRID_ELEMENT_SIZE: [f32; 2] = [80.0, 80.0];
pub const PRESET_GRID_ELEMENT_SIZE_MIN: [f32; 2] = [60.0, 60.0];

mod button;
mod row;

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut DemexUiContext) {
    let preset_handler = context.preset_handler.read();
    let updatable_handler = context.updatable_handler.read();

    let _selected_fixtures = context
        .global_fixture_select
        .as_ref()
        .map(|selection| selection.fixtures().to_vec());
    let selected_fixtures = _selected_fixtures.as_ref();

    let min_num_preset_buttons = 0;

    ui.vertical(|ui| {
        // Groups
        preset_grid_row_ui(
            ui,
            "Groups",
            None,
            ecolor::Color32::DARK_RED,
            |ui, buttons_per_row, size_overhead| {
                for id in 0..=preset_handler.next_group_id().max(min_num_preset_buttons) {
                    if id != 0 && id % buttons_per_row as u32 == 0 {
                        ui.end_row();
                    }

                    let g = preset_handler.get_group(id);

                    let config = if let Ok(g) = g {
                        PresetGridButtonConfig::Preset {
                            id: g.id(),
                            name: g.name().to_owned(),
                            top_bar_color: context.global_fixture_select.as_ref().and_then(
                                |selection| {
                                    if selection == g.fixture_selection() {
                                        Some(ecolor::Color32::GREEN)
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
                    .show(ui, size_overhead);

                    if response.clicked()
                        || quick_action
                            .is_some_and(|a| a == PresetGridButtonQuickMenuActions::Default)
                    {
                        if let Ok(g) = g {
                            context
                                .action_queue
                                .enqueue_now(Action::InternalSetFixtureSelection(Some(
                                    g.fixture_selection().clone(),
                                )));
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
            },
        );

        // Feature Presets
        for feature_group in FixtureChannel3FeatureGroup::iter() {
            preset_grid_row_ui(
                ui,
                feature_group.name(),
                Some(feature_group.into()),
                ecolor::Color32::BLUE,
                |ui, buttons_per_row, size_overhead| {
                    for id in 0..=preset_handler
                        .next_preset_id(feature_group)
                        .max(min_num_preset_buttons)
                    {
                        if id != 0 && id % buttons_per_row as u32 == 0 {
                            ui.end_row();
                        }

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
                        .show(ui, size_overhead);

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
                                            DemexEditWindow::EditPreset2(p.id()),
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
                                context.action_queue.enqueue_now(Action::SetFixturePreset(
                                    SetFixturePresetArgs {
                                        selection_or_selector: SelectionOrSelector::Selection(
                                            selection.clone(),
                                        ),
                                        preset_id: ValueOrRange::Single(preset_id),
                                    },
                                ));
                            }
                        }
                    }
                },
            );
        }

        // Macros
        preset_grid_row_ui(
            ui,
            "Macros",
            None,
            ecolor::Color32::BROWN,
            |ui, buttons_per_row, size_overhead| {
                for id in 0..=preset_handler.next_macro_id().max(min_num_preset_buttons) {
                    if id != 0 && id % buttons_per_row as u32 == 0 {
                        ui.end_row();
                    }

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

                    let (response, quick_action) = PresetGridButton::new(
                        config,
                        PresetGridButtonDecoration::default(),
                        None,
                        None,
                    )
                    .show(ui, size_overhead);

                    if response.clicked()
                        || quick_action.is_some_and(|action| {
                            action == PresetGridButtonQuickMenuActions::Default
                        })
                    {
                        if let Ok(m) = m {
                            context.action_queue.enqueue_now(m.action().clone());
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
                                context
                                    .window_handler
                                    .add_window(DemexWindow::Edit(DemexEditWindow::EditMacro(id)));
                            }
                            _ => {}
                        }
                    }
                }
            },
        );

        // Command slices
        preset_grid_row_ui(
            ui,
            "Command Slices",
            None,
            ecolor::Color32::GOLD,
            |ui, buttons_per_row, size_overhead| {
                for id in 0..=preset_handler
                    .next_command_slice_id()
                    .max(min_num_preset_buttons)
                {
                    if id != 0 && id % buttons_per_row as u32 == 0 {
                        ui.end_row();
                    }

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

                    let (response, quick_action) = PresetGridButton::new(
                        config,
                        PresetGridButtonDecoration::default(),
                        None,
                        None,
                    )
                    .show(ui, size_overhead);

                    if response.clicked()
                        || quick_action.is_some_and(|action| {
                            action == PresetGridButtonQuickMenuActions::Default
                        })
                    {
                        if let Ok(cs) = cs {
                            context.command.extend_from_slice(cs.command());
                        }
                    }
                }
            },
        );

        // Executors
        preset_grid_row_ui(
            ui,
            "Executors",
            None,
            ecolor::Color32::DARK_GREEN,
            |ui, max_buttons_per_row, size_overhead| {
                for id in 0..=updatable_handler
                    .next_executor_id()
                    .max(min_num_preset_buttons)
                {
                    if id != 0 && id % max_buttons_per_row as u32 == 0 {
                        ui.end_row();
                    }

                    let executor_exists = updatable_handler.executor(id).is_ok();

                    let (config, decoration) = if executor_exists {
                        let executor = updatable_handler.executor(id).unwrap();

                        let config = PresetGridButtonConfig::Preset {
                            id,
                            name: executor.display_name(&preset_handler),
                            top_bar_color: if updatable_handler.executor(id).unwrap().is_active() {
                                Some(ecolor::Color32::RED)
                            } else {
                                None
                            },
                            display_color: None,
                        };

                        let decoration = {
                            let executor = updatable_handler.executor(id).unwrap();

                            let current_cues = executor.runtime().current_cues();
                            let current_cue_text = if current_cues.is_empty() {
                                "-".to_owned()
                            } else {
                                current_cues
                                    .into_iter()
                                    .map(|c| (c + 1).to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            };

                            PresetGridButtonDecoration {
                                right_top_text: Some(format!(
                                    "({})/{}",
                                    current_cue_text,
                                    executor.runtime().num_cues(&preset_handler)
                                )),
                                left_bottom_text: Some("Seq".to_owned()),
                            }
                        };

                        (config, decoration)
                    } else {
                        (
                            PresetGridButtonConfig::Empty { id },
                            PresetGridButtonDecoration::default(),
                        )
                    };

                    let (response, quick_action) = PresetGridButton::new(
                        config,
                        decoration,
                        Some(vec![
                            PresetGridButtonQuickMenuActions::Custom("Stop"),
                            PresetGridButtonQuickMenuActions::Custom("Edit Sequence"),
                            PresetGridButtonQuickMenuActions::Custom("Insert Sequence"),
                        ]),
                        None,
                    )
                    .show(ui, size_overhead);

                    if response.clicked()
                        || quick_action
                            .is_some_and(|a| a == PresetGridButtonQuickMenuActions::Default)
                    {
                        if executor_exists {
                            context.action_queue.enqueue_now(Action::InternalExecutorGo(
                                ExecutorGoArgs { executor_id: id },
                            ));
                        } else {
                            context.command.extend_from_slice(&[
                                Token::KeywordRecord,
                                Token::KeywordExecutor,
                                Token::Integer(id),
                            ]);
                        }
                    }

                    if let Some(quick_action) = quick_action {
                        match quick_action {
                            PresetGridButtonQuickMenuActions::Insert => {
                                context.command.extend_from_slice(&[
                                    Token::KeywordExecutor,
                                    Token::Integer(id),
                                ]);
                            }

                            PresetGridButtonQuickMenuActions::New => {
                                context.command.extend_from_slice(&[
                                    Token::KeywordRecord,
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
                            PresetGridButtonQuickMenuActions::Custom("Edit Sequence") => {
                                context.global_sequence_select = UiEditRequest::Request(
                                    updatable_handler
                                        .executor(id)
                                        .unwrap()
                                        .runtime()
                                        .sequence_id(),
                                );
                            }
                            PresetGridButtonQuickMenuActions::Custom("Insert Sequence") => {
                                context.command.extend_from_slice(&[
                                    Token::KeywordSequence,
                                    Token::Integer(
                                        updatable_handler
                                            .executor(id)
                                            .unwrap()
                                            .runtime()
                                            .sequence_id(),
                                    ),
                                ]);
                            }
                            _ => {}
                        }
                    }

                    if (response.secondary_clicked()
                        || quick_action
                            .is_some_and(|a| a == PresetGridButtonQuickMenuActions::Custom("Stop")))
                        && executor_exists
                        && updatable_handler.executor(id).unwrap().is_active()
                    {
                        context
                            .action_queue
                            .enqueue_now(Action::InternalExecutorStop(ExecutorStopArgs {
                                executor_id: id,
                            }));
                    }
                }
            },
        );
    });
}
