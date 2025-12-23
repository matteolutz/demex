use std::collections::HashMap;

use crate::{
    engine::component::Component,
    fixture::FixturePath,
    patch::Patch,
    pool::{PoolItem, PoolType},
    selection::FixtureSelection,
    state::fixture_state::FixtureState,
};

#[derive(Debug, Clone, Default)]
pub struct DemexEngineState {
    pub fixture_selection: Option<FixtureSelection>,
}

impl Component for DemexEngineState {}

#[derive(Debug)]
pub struct DemexFrontendInitState {
    pub fixture_selection: Option<FixtureSelection>,
    pub fixture_states: HashMap<FixturePath, FixtureState>,
    pub pools: HashMap<PoolType, Vec<PoolItem>>,
    pub patch: Patch,
}
