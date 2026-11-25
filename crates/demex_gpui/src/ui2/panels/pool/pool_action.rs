use demex_core::{
    command::{
        lexer::token::Token,
        parser::{
            expected::ExpectedParseSlice,
            nodes::{
                action::{
                    Action,
                    functions::{
                        create_function::CreateEffectPresetArgs, delete_function::DeleteArgs,
                        go_function::ExecutorGoArgs, set_function::SetFixturePresetArgs,
                        stop_function::ExecutorStopArgs,
                    },
                },
                fixture_selector::FixtureSelector,
                object::{HomeableObject, Object, ObjectRange},
            },
        },
    },
    engine::comm::ExecutorSequenceRequest,
    has_flag,
    pool::{PoolItem, PoolType},
    presets::{
        group::FixtureGroupProperty,
        preset::{FixturePresetFlags, FixturePresetId, FixturePresetProperty},
    },
    sequence::{SequenceProperty, frontend::FrontendSequence},
};
use gpui::{App, PromptButton, prelude::FluentBuilder};

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::{
        panels::pool::pool_button::PoolButton,
        window::{
            edit_keyframe_effect::EditKeyframeEffectWindow,
            set_property::{SetPropertyWindow, SetPropertyWindowPropertyType},
        },
        wm::{WindowManager, edit_window::WindowManagerExtension},
    },
};

pub fn append_pool_item_to_command(pool_type: PoolType, pool_item_id: u32, cx: &mut App) {
    DemexUiState::update_command_input_state(cx, |state, cx| {
        state.append_pool_item(pool_type, pool_item_id, cx);
    });
}

pub fn handle_pool_item_click(
    pool_type: PoolType,
    pool_item_id: u32,
    pool_item_exists: bool,
    cx: &mut App,
) {
    let engine = DemexEngineHandler::engine(cx);
    let command = DemexUiState::command_input_state(cx).map(|state| state.value(cx));

    if let Some(cmd) = command.and_then(|cmd| (!cmd.is_empty()).then_some(cmd)) {
        if let Some(parse_error) = engine.parse_command(&cmd).err() {
            if parse_error.was_expected(ExpectedParseSlice::Object(pool_type.into())) {
                append_pool_item_to_command(pool_type, pool_item_id, cx);
                return;
            }
        }
    }

    if !pool_item_exists {
        return;
    }

    match pool_type {
        PoolType::Preset(preset_type) => engine.exec_ui(Action::SetFixturePreset(
            SetFixturePresetArgs::current(FixturePresetId::new(preset_type, pool_item_id)),
        )),
        PoolType::Executor => engine.exec_ui(Action::ExecutorGo(ExecutorGoArgs {
            executor_id: pool_item_id,
        })),
        PoolType::Sequence => {
            DemexUiState::selected_sequence(cx).update(cx, |selected_sequence, cx| {
                *selected_sequence = Some(pool_item_id);
                cx.notify();
            })
        }
        PoolType::Group => engine.exec_ui(Action::FixtureSelector(FixtureSelector::group(
            pool_item_id,
        ))),
        PoolType::Macro => engine.exec_ui(Action::RunMacro(pool_item_id)),
        _ => {}
    }
}

pub fn apply_pool_type_to_button(
    pool_type: PoolType,
    button: PoolButton,
    pool_item: Option<&PoolItem>,
    pool_item_id: u32,
) -> PoolButton {
    match pool_type {
        PoolType::Executor => {
            fn get_sequence(
                executor_id: u32,
                cx: &mut App,
                cb: impl FnOnce(Option<FrontendSequence>, &mut App) + Send + 'static,
            ) {
                DemexEngineHandler::send(cx, ExecutorSequenceRequest { executor_id }, cb)
            }

            button
                .when(pool_item.is_some(), |this| {
                    this.action_at(0, "Name", move |_, cx| {
                        get_sequence(pool_item_id, cx, |seq, cx| {
                            if let Some(seq) = seq {
                                WindowManager::open_edit_window::<SetPropertyWindow>(
                                    cx,
                                    move |window, cx| {
                                        SetPropertyWindow::new(
                                            Object::Sequence(seq.id),
                                            SequenceProperty::Name,
                                            SetPropertyWindowPropertyType::String,
                                            window,
                                            cx,
                                        )
                                    },
                                );
                            }
                        });
                    })
                })
                .action_at(1, "Insert", move |_, cx| {
                    DemexUiState::update_command_input_state(cx, |state, cx| {
                        state.append_pool_item(PoolType::Executor, pool_item_id, cx);
                    });
                })
                .when(pool_item.is_some(), |this| {
                    this.action_at(2, "Edit Seq", move |_, cx| {
                        get_sequence(pool_item_id, cx, |seq, cx| {
                            if let Some(seq) = seq {
                                DemexUiState::selected_sequence(cx).update(
                                    cx,
                                    |selected_sequence, cx| {
                                        *selected_sequence = Some(seq.id);
                                        cx.notify();
                                    },
                                );
                            }
                        });
                    })
                    .action_at(3, "Stop", move |_, cx| {
                        DemexEngineHandler::engine(cx).exec_ui(Action::ExecutorStop(
                            ExecutorStopArgs {
                                executor_id: pool_item_id,
                            },
                        ))
                    })
                    .action_at(4, "Del", move |window, cx| {
                        let answer = window.prompt(
                            gpui::PromptLevel::Warning,
                            format!("Delete Executor {}", pool_item_id).as_str(),
                            Some("Do you really want to delete this executor?"),
                            &[PromptButton::ok("Yes"), PromptButton::cancel("No")],
                            cx,
                        );

                        cx.spawn(async move |cx| {
                            let Ok(answer) = answer.await else {
                                return;
                            };

                            // Yes
                            if answer == 0 {
                                cx.update(|cx| {
                                    DemexEngineHandler::engine(cx).exec_ui(Action::Delete(
                                        DeleteArgs {
                                            object_range: ObjectRange::single(
                                                Object::HomeableObject(HomeableObject::Executor(
                                                    pool_item_id,
                                                )),
                                            ),
                                        },
                                    ));
                                });
                            }
                        })
                        .detach();
                    })
                    .action_at(5, "Insert Seq", move |_, cx| {
                        get_sequence(pool_item_id, cx, |seq, cx| {
                            if let Some(seq) = seq {
                                DemexUiState::update_command_input_state(cx, |state, cx| {
                                    state.append_pool_item(PoolType::Sequence, seq.id, cx);
                                });
                            }
                        });
                    })
                    .action_at(6, "Assign", move |_, cx| {
                        DemexUiState::update_command_input_state(cx, |state, cx| {
                            state.append(
                                format!("assign {} {}", Token::KeywordExecutor, pool_item_id),
                                cx,
                            )
                        });
                    })
                })
        }
        PoolType::Preset(feature_group) => button
            .when(
                pool_item.is_some_and(|item| has_flag!(item, FixturePresetFlags::FeatureEffect)),
                |this| this.top_right("FeFx"),
            )
            .when(pool_item.is_some(), |this| {
                this.action_at(0, "Name", move |_, cx| {
                    WindowManager::open_edit_window::<SetPropertyWindow>(cx, move |window, cx| {
                        SetPropertyWindow::new(
                            Object::Preset(FixturePresetId {
                                feature_group,
                                preset_id: pool_item_id,
                            }),
                            FixturePresetProperty::Name,
                            SetPropertyWindowPropertyType::String,
                            window,
                            cx,
                        )
                    });
                })
                .action_at(4, "Del", move |window, cx| {
                    let preset_id = FixturePresetId {
                        feature_group,
                        preset_id: pool_item_id,
                    };

                    let answer = window.prompt(
                        gpui::PromptLevel::Warning,
                        format!("Delete {}", preset_id).as_str(),
                        Some("Do you really want to delete this preset?"),
                        &[PromptButton::ok("Yes"), PromptButton::cancel("No")],
                        cx,
                    );

                    cx.spawn(async move |cx| {
                        let Ok(answer) = answer.await else {
                            return;
                        };

                        // Yes
                        if answer == 0 {
                            cx.update(|cx| {
                                DemexEngineHandler::engine(cx).exec_ui(Action::Delete(
                                    DeleteArgs {
                                        object_range: ObjectRange::single(Object::Preset(
                                            preset_id,
                                        )),
                                    },
                                ));
                            });
                        }
                    })
                    .detach();
                })
            })
            .action_at(1, "Insert", move |_, cx| {
                DemexUiState::update_command_input_state(cx, |state, cx| {
                    state.append_pool_item(PoolType::Preset(feature_group), pool_item_id, cx);
                });
            })
            .when_none(&pool_item, |this| {
                this.action_at(3, "Crt FX", move |_, cx| {
                    DemexEngineHandler::engine(cx).exec_ui(Action::CreateEffectPreset(
                        CreateEffectPresetArgs {
                            id: FixturePresetId {
                                feature_group,
                                preset_id: pool_item_id,
                            },
                            name: None,
                        },
                    ));
                })
            })
            .when(
                pool_item.is_some_and(|item| has_flag!(item, FixturePresetFlags::KeyframeEffect)),
                |this| {
                    this.top_right("KFx").action("Edit FX", move |_, cx| {
                        WindowManager::open_edit_window::<EditKeyframeEffectWindow>(
                            cx,
                            move |window, cx| {
                                EditKeyframeEffectWindow::new(
                                    (feature_group, pool_item_id),
                                    window,
                                    cx,
                                )
                            },
                        );
                    })
                },
            ),
        PoolType::Group => button.when(pool_item.is_some(), |this| {
            this.action_at(0, "Name", move |_, cx| {
                WindowManager::open_edit_window::<SetPropertyWindow>(cx, move |window, cx| {
                    SetPropertyWindow::new(
                        HomeableObject::Group(pool_item_id),
                        FixtureGroupProperty::Name,
                        SetPropertyWindowPropertyType::String,
                        window,
                        cx,
                    )
                });
            })
            .action_at(1, "Insert", move |_, cx| {
                DemexUiState::update_command_input_state(cx, |state, cx| {
                    state.append_pool_item(PoolType::Group, pool_item_id, cx)
                });
            })
            .action_at(4, "Del", move |window, cx| {
                let answer = window.prompt(
                    gpui::PromptLevel::Warning,
                    format!("Delete Group {}", pool_item_id).as_str(),
                    Some("Do you really want to delete this group?"),
                    &[PromptButton::ok("Yes"), PromptButton::cancel("No")],
                    cx,
                );

                cx.spawn(async move |cx| {
                    let Ok(answer) = answer.await else {
                        return;
                    };

                    // Yes
                    if answer == 0 {
                        cx.update(|cx| {
                            DemexEngineHandler::engine(cx).exec_ui(Action::Delete(DeleteArgs {
                                object_range: ObjectRange::single(Object::group(pool_item_id)),
                            }));
                        });
                    }
                })
                .detach();
            })
        }),
        _ => button,
    }
}
