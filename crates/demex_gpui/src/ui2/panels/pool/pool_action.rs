use demex_core::{
    command::{
        lexer::token::Token,
        parser::nodes::{
            action::{
                Action,
                functions::{go_function::ExecutorGoArgs, set_function::SetFixturePresetArgs},
            },
            fixture_selector::FixtureSelector,
            object::{HomeableObject, Object},
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
use gpui::{App, prelude::FluentBuilder};

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::{
        panels::pool::pool_button::PoolButton,
        window::{
            edit_keyframe_effect::EditKeyframeEffectWindow,
            set_property::{SetPropertyWindow, SetPropertyWindowPropertyType},
        },
        wm::{WindowManager, app::WindowManagerAppExt, edit_window::WindowManagerExtension},
    },
};

pub fn handle_pool_item_click(pool_type: PoolType, pool_item_id: u32, cx: &mut App) {
    let engine = DemexEngineHandler::engine(cx);

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
                .action("Name", move |_, cx| {
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
                .action("Insert", move |_, cx| {
                    cx.defer(move |cx| {
                        cx.update_wm(|wm, cx| {
                            let _ = wm.update_main_dock_window(cx, |dock_window, window, cx| {
                                dock_window.append_to_command(
                                    format!("{} {}", Token::KeywordExecutor, pool_item_id),
                                    window,
                                    cx,
                                );
                            });
                        });
                    });
                })
                .action("Edit Seq", move |_, cx| {
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
        }
        PoolType::Preset(feature_group) => button
            .when(
                pool_item.is_some_and(|item| has_flag!(item, FixturePresetFlags::FeatureEffect)),
                |this| this.top_right("FeFx"),
            )
            .action("Name", move |_, cx| {
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
            .action("Insert", move |_, cx| {
                cx.defer(move |cx| {
                    cx.update_wm(|wm, cx| {
                        let _ = wm.update_main_dock_window(cx, |dock_window, window, cx| {
                            dock_window.append_to_command(
                                format!(
                                    "{} {}.{}",
                                    Token::KeywordPreset,
                                    feature_group as u32,
                                    pool_item_id
                                ),
                                window,
                                cx,
                            );
                        });
                    });
                });
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
        PoolType::Group => button.action("Name", move |_, cx| {
            WindowManager::open_edit_window::<SetPropertyWindow>(cx, move |window, cx| {
                SetPropertyWindow::new(
                    HomeableObject::Group(pool_item_id),
                    FixtureGroupProperty::Name,
                    SetPropertyWindowPropertyType::String,
                    window,
                    cx,
                )
            });
        }),
        _ => button,
    }
}
