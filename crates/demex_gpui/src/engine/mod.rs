/*
 *
 * This file has been modified from its original version.
 * Original: https://github.com/BaukeWestendorp/radiant
 * License: Apache 2.0 - https://github.com/BaukeWestendorp/radiant/blob/main/LICENCE
 *
 */

use std::sync::mpsc;

use demex_core::{
    engine::{DemexEngine, error::DemexEngineError},
    show::DemexShow,
};
use gdtf::fixture_type::FixtureType;
use gpui::{App, AppContext, Entity, Global};

use crate::engine::event::DemexEventHandler;

pub mod event;

pub struct DemexEngineHandler {
    engine: DemexEngine,
    event_handler: Entity<DemexEventHandler>,
}

impl DemexEngineHandler {
    pub fn init(
        global_fixture_types: Vec<FixtureType>,
        show: DemexShow,
        cx: &mut App,
    ) -> Result<(), DemexEngineError> {
        let (event_bus_tx, event_bus_rx) = mpsc::channel();

        let mut engine = DemexEngine::new(event_bus_tx);

        let event_handler = cx.new(|cx| DemexEventHandler::new(event_bus_rx, cx));

        engine.load_show(show, global_fixture_types, true);

        cx.set_global(Self {
            engine,
            event_handler,
        });

        Ok(())
    }

    pub fn engine(cx: &App) -> &DemexEngine {
        let this: &Self = cx.global();
        &this.engine
    }

    pub fn event_handler(cx: &App) -> Entity<DemexEventHandler> {
        let this: &Self = cx.global();
        this.event_handler.clone()
    }
}

impl Global for DemexEngineHandler {}
