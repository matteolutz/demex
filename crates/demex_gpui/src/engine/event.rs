/*
 *
 * This file has been modified from its original version.
 * Original: https://github.com/BaukeWestendorp/radiant
 * License: Apache 2.0 - https://github.com/BaukeWestendorp/radiant/blob/main/LICENCE
 *
 */

use gpui::{AsyncApp, Entity, EventEmitter, Task, Timer, prelude::*};
use gpui_component::{button::Button, notification::Notification};
use std::{sync::mpsc, time::Duration};

use demex_core::{
    command::parser::nodes::action::result::ActionRunResult, engine::comm::DemexEngineCommEvent,
    event::DemexEvent,
};

use crate::{
    engine::{showfile::DemexShowFileManager, state::DemexUiState},
    ui2::wm::{WindowManager, app::WindowManagerAsyncAppExt},
};

pub struct DemexEventHandler {
    _tasks: Vec<Task<()>>,
}

impl EventEmitter<DemexEvent> for DemexEventHandler {}

impl DemexEventHandler {
    pub fn new(
        event_rx: mpsc::Receiver<DemexEngineCommEvent>,
        cx: &mut gpui::Context<Self>,
    ) -> Self {
        let _tasks = vec![cx.spawn(async move |event_handler, cx| {
            loop {
                Timer::after(Duration::from_millis(16)).await;

                let Some(event_handler) = event_handler.upgrade() else {
                    continue;
                };

                for event in event_rx.try_iter() {
                    Self::handle_comm_event(&event_handler, event, cx);
                }
            }
        })];

        Self { _tasks }
    }
}

impl DemexEventHandler {
    fn handle_comm_event(
        event_handler: &Entity<Self>,
        event: DemexEngineCommEvent,
        cx: &mut AsyncApp,
    ) {
        match event {
            DemexEngineCommEvent::DemexEvent(event) => Self::handle_event(event_handler, event, cx),
            DemexEngineCommEvent::Error(err) => {
                let _ = cx.update_wm(|wm, cx| wm.push_notifcation(Notification::error(err), cx));
            }
            DemexEngineCommEvent::ActionRunResult(result) => {
                Self::handle_action_run_result(result, cx)
            }
            DemexEngineCommEvent::TickStateUpdate(tick_state) => {
                cx.update_global(|ui_state: &mut DemexUiState, cx| {
                    ui_state.update_from_tick(tick_state, cx);
                });
            }
            DemexEngineCommEvent::FixtureValuesUpdate(fixture_values) => {
                cx.update_global(|ui_state: &mut DemexUiState, cx| {
                    ui_state.update_fixture_values(fixture_values, cx);
                });
            }
        }
    }

    fn handle_event(event_handler: &Entity<Self>, event: DemexEvent, cx: &mut AsyncApp) {
        cx.update_global(|ui_state: &mut DemexUiState, cx| {
            ui_state.update_from_event(event.clone(), cx);
        });

        cx.update_entity(&event_handler, |_, cx| cx.emit(event));
    }

    fn handle_action_run_result(result: ActionRunResult, cx: &mut AsyncApp) {
        match result {
            ActionRunResult::Info(info) => {
                let _ = cx.update_wm(|wm, cx| wm.push_notifcation(Notification::info(info), cx));
            }
            ActionRunResult::Warn(warn) => {
                let _ = cx.update_wm(|wm, cx| wm.push_notifcation(Notification::warning(warn), cx));
            }
            ActionRunResult::Save => {
                let _ = cx.update(|cx| DemexShowFileManager::save(None, cx, |_, _| {}));
            }
            ActionRunResult::UpdatePatch(patch) => {
                let _ = cx.update_global(|ui_state: &mut DemexUiState, cx| {
                    ui_state.update_patch(patch, cx);
                });
                let _ = cx.update_wm(|wm, cx| {
                    wm.push_notifcation(
                        Notification::info("Patch updated").action(|_, _, _| {
                            Button::new("reload").label("Reload").on_click(|_, _, cx| {
                                let _ = WindowManager::inspect_error(
                                    DemexShowFileManager::reload(cx),
                                    "Failed to reload: ",
                                    cx,
                                );
                            })
                        }),
                        cx,
                    )
                });
            }
            _ => {}
        }
    }
}
