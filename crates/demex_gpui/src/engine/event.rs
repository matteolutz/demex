/*
 *
 * This file has been modified from its original version.
 * Original: https://github.com/BaukeWestendorp/radiant
 * License: Apache 2.0 - https://github.com/BaukeWestendorp/radiant/blob/main/LICENCE
 *
 */

use gpui::{EventEmitter, Timer, prelude::*};
use std::{sync::mpsc, time::Duration};

use demex_core::event::DemexEvent;

pub struct DemexEventHandler {}

impl EventEmitter<DemexEvent> for DemexEventHandler {}

impl DemexEventHandler {
    pub fn new(event_rx: mpsc::Receiver<DemexEvent>, cx: &mut gpui::Context<Self>) -> Self {
        cx.spawn(async move |event_handler, cx| {
            loop {
                Timer::after(Duration::from_millis(16)).await;

                let Some(event_handler) = event_handler.upgrade() else {
                    continue;
                };

                let Ok(event) = event_rx.try_recv() else {
                    continue;
                };

                cx.update_entity(&event_handler, |_, cx| cx.emit(event))
                    .unwrap();
            }
        })
        .detach();

        Self {}
    }
}
