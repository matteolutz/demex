/*
 *
 * This file has been modified from its original version.
 * Original: https://github.com/BaukeWestendorp/radiant
 * License: Apache 2.0 - https://github.com/BaukeWestendorp/radiant/blob/main/LICENCE
 *
 */

use gpui::{EventEmitter, Timer, prelude::*};
use gpui_component::notification::Notification;
use std::{sync::mpsc, time::Duration};

use demex_core::{
    command::parser::nodes::action::result::ActionRunResult,
    engine::comm::{DemexEngineCommEvent, ShowRequest},
    event::DemexEvent,
};

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::wm::app::WindowManagerAsyncAppExt,
};

pub struct DemexEventHandler {}

impl EventEmitter<DemexEvent> for DemexEventHandler {}

impl DemexEventHandler {
    pub fn new(
        event_rx: mpsc::Receiver<DemexEngineCommEvent>,
        cx: &mut gpui::Context<Self>,
    ) -> Self {
        cx.spawn(async move |event_handler, cx| {
            loop {
                Timer::after(Duration::from_millis(16)).await;

                let Some(event_handler) = event_handler.upgrade() else {
                    continue;
                };

                for event in event_rx.try_iter() {
                    match event {
                        DemexEngineCommEvent::DemexEvent(event) => {
                            cx.update_global(|ui_state: &mut DemexUiState, cx| {
                                ui_state.update_from_event(event.clone(), cx);
                            })
                            .unwrap();

                            cx.update_entity(&event_handler, |_, cx| cx.emit(event))
                                .unwrap();
                        }
                        DemexEngineCommEvent::Error(err) => {
                            let _ = cx.update_wm(|wm, cx| {
                                wm.push_notifcation(Notification::error(err), cx)
                            });
                        }
                        DemexEngineCommEvent::ActionRunResult(result) => match result {
                            ActionRunResult::Info(info) => {
                                let _ = cx.update_wm(|wm, cx| {
                                    wm.push_notifcation(Notification::info(info), cx)
                                });
                            }
                            ActionRunResult::Warn(warn) => {
                                let _ = cx.update_wm(|wm, cx| {
                                    wm.push_notifcation(Notification::warning(warn), cx)
                                });
                            }
                            ActionRunResult::Save => {
                                let _ = cx.update(|cx| {
                                    DemexEngineHandler::send(cx, ShowRequest {}, |show, _| {
                                        // TODO
                                        println!("saving show: {:?}", show);
                                    })
                                });
                            }
                            ActionRunResult::WithEvent { .. } => unreachable!(),
                            _ => {}
                        },
                        DemexEngineCommEvent::TickStateUpdate(tick_state) => {
                            cx.update_global(|ui_state: &mut DemexUiState, cx| {
                                ui_state.update_from_tick(tick_state, cx);
                            })
                            .unwrap();
                        }
                        DemexEngineCommEvent::FixtureValuesUpdate(fixture_values) => {
                            cx.update_global(|ui_state: &mut DemexUiState, cx| {
                                ui_state.update_fixture_values(fixture_values, cx);
                            })
                            .unwrap();
                        }
                    }
                }
            }
        })
        .detach();

        Self {}
    }
}
