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
    fixture::GdtfFixture,
    patch::Patch,
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
        show.register(global_fixture_types, &mut engine);

        let event_handler = cx.new(|cx| DemexEventHandler::new(event_bus_rx, cx));

        engine.start(false);

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

    pub fn read_fixture<R>(
        cx: &App,
        fixture_id: u32,
        f: impl FnOnce(&GdtfFixture) -> R,
    ) -> Option<R> {
        Self::engine(cx)
            .fixture_handler()
            .read(|fh| fh.fixture_immut(fixture_id).map(|fixture| f(fixture)))
    }

    pub fn read_fixture_and_patch<R>(
        cx: &App,
        fixture_id: u32,
        f: impl FnOnce(&GdtfFixture, &Patch) -> R,
    ) -> Option<R> {
        Self::engine(cx).fixture_handler().read(|fh| {
            Self::engine(cx).patch().read(|patch| {
                fh.fixture_immut(fixture_id)
                    .map(|fixture| f(fixture, patch))
            })
        })
    }
}

impl Global for DemexEngineHandler {}
