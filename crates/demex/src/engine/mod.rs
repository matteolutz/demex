/*
 *
 * This file has been modified from its original version.
 * Original: https://github.com/BaukeWestendorp/radiant
 * License: Apache 2.0 - https://github.com/BaukeWestendorp/radiant/blob/main/LICENCE
 *
 */

use demex_core::{
    engine::{DemexEngine, error::DemexEngineError},
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

    pub fn engine(&self) -> &DemexEngine {
        &self.engine
    }
}

impl Global for DemexEngineHandler {}
