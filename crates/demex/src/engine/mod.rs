/*
 *
 * This file has been modified from its original version.
 * Original: https://github.com/BaukeWestendorp/radiant
 * License: Apache 2.0 - https://github.com/BaukeWestendorp/radiant/blob/main/LICENCE
 *
 */

use demex_core::{
    engine::{DemexEngine, error::DemexEngineError},
    fixture::GdtfFixture,
    show::DemexShow,
};
use gdtf::fixture_type::FixtureType;
use gpui::{App, Global};

pub struct DemexEngineHandler {
    engine: DemexEngine,
}

impl DemexEngineHandler {
    pub fn init(
        global_fixture_types: Vec<FixtureType>,
        show: DemexShow,
        cx: &mut App,
    ) -> Result<(), DemexEngineError> {
        let mut engine = DemexEngine::new();
        show.register(global_fixture_types, &mut engine);

        engine.start();

        cx.set_global(Self { engine });

        Ok(())
    }

    pub fn engine(cx: &App) -> &DemexEngine {
        let this: &Self = cx.global();
        &this.engine
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
}

impl Global for DemexEngineHandler {}
