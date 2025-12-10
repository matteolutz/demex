use std::collections::HashMap;

use crate::{
    engine::component::Component,
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
    pub fixture_states: HashMap<u32, FixtureState>,
    pub pools: HashMap<PoolType, Vec<PoolItem>>,
    pub patch: Patch,
}
